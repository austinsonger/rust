use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{debug, warn};

use crate::{
    constants::rate_limit::{DEFAULT_MAX_REQUESTS, DEFAULT_WINDOW_SECONDS},
    errors::Error,
};

/// A simple in-memory rate limiter
/// In a production environment, this would be replaced with a distributed rate limiter
/// using Redis or a similar technology
#[derive(Debug, Clone)]
pub struct RateLimiter {
    // Map of IP address to a list of request timestamps
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    // Maximum number of requests allowed in the window
    max_requests: u32,
    // Window duration in seconds
    window_seconds: u64,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }
    
    /// Create a new rate limiter with default settings
    pub fn default() -> Self {
        Self::new(DEFAULT_MAX_REQUESTS, DEFAULT_WINDOW_SECONDS)
    }
    
    /// Check if a request from the given IP address is allowed
    pub fn is_allowed(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_seconds);
        
        let mut requests = self.requests.lock().unwrap();
        
        // Get or create the list of request timestamps for this IP
        let timestamps = requests.entry(ip).or_insert_with(Vec::new);
        
        // Remove timestamps that are outside the window
        timestamps.retain(|&timestamp| now.duration_since(timestamp) < window);
        
        // Check if the number of requests is below the limit
        if timestamps.len() as u32 >= self.max_requests {
            return false;
        }
        
        // Add the current timestamp
        timestamps.push(now);
        
        true
    }
    
    /// Clean up old entries
    pub fn cleanup(&self) {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_seconds);
        
        let mut requests = self.requests.lock().unwrap();
        
        // Remove entries with no timestamps in the window
        requests.retain(|_, timestamps| {
            timestamps.retain(|&timestamp| now.duration_since(timestamp) < window);
            !timestamps.is_empty()
        });
    }
}

/// Extract the client IP address from the request
fn get_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // Try to get the IP from the X-Forwarded-For header
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_for) = forwarded_for.to_str() {
            if let Some(ip) = forwarded_for.split(',').next() {
                if let Ok(ip) = ip.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }
    
    // Try to get the IP from the X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(real_ip) = real_ip.to_str() {
            if let Ok(ip) = real_ip.trim().parse() {
                return Some(ip);
            }
        }
    }
    
    // Default to a placeholder IP if we can't determine the real one
    // In a real application, you would want to handle this differently
    Some("127.0.0.1".parse().unwrap())
}

/// Middleware to apply rate limiting
pub fn rate_limit(
    rate_limiter: RateLimiter,
) -> impl Fn(Request, Next) -> Response + Clone {
    move |request: Request, next: Next| async move {
        // Extract the client IP address
        let ip = match get_client_ip(request.headers()) {
            Some(ip) => ip,
            None => {
                warn!("Could not determine client IP address");
                return (
                    StatusCode::BAD_REQUEST,
                    "Could not determine client IP address",
                )
                    .into_response();
            }
        };
        
        // Check if the request is allowed
        if !rate_limiter.is_allowed(ip) {
            warn!(
                ip = %ip,
                max_requests = rate_limiter.max_requests,
                window_seconds = rate_limiter.window_seconds,
                "Rate limit exceeded"
            );
            
            return Error::RateLimitExceeded.into_response();
        }
        
        debug!(
            ip = %ip,
            "Request allowed by rate limiter"
        );
        
        // Continue with the request
        next.run(request).await
    }
}

/// Middleware to apply rate limiting for login attempts
pub fn login_rate_limit(
    rate_limiter: RateLimiter,
) -> impl Fn(Request, Next) -> Response + Clone {
    rate_limit(rate_limiter)
}

/// Middleware to apply rate limiting for API requests
pub fn api_rate_limit(
    rate_limiter: RateLimiter,
) -> impl Fn(Request, Next) -> Response + Clone {
    rate_limit(rate_limiter)
}
