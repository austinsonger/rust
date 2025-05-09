use axum::{
    extract::{Path, Query},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    constants::rate_limit::{DEFAULT_MAX_REQUESTS, DEFAULT_WINDOW_SECONDS},
    errors::Error,
    middleware::{
        auth::require_admin,
        rate_limiter::{RateLimiter, api_rate_limit},
    },
    utils::{
        authenticate_request::TokenUser,
        response_formatter,
    },
};

pub fn create_route() -> Router {
    // Create a rate limiter for admin routes
    let admin_rate_limiter = RateLimiter::new(
        DEFAULT_MAX_REQUESTS / 2, // More restrictive rate limit for admin routes
        DEFAULT_WINDOW_SECONDS,
    );
    
    Router::new()
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/users", get(list_users))
        .route("/admin/system/status", get(system_status))
        // Apply middleware to all routes
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn_with_state(
            admin_rate_limiter,
            api_rate_limit,
        ))
}

#[derive(Debug, Serialize)]
struct AdminDashboard {
    total_users: u64,
    total_products: u64,
    total_orders: u64,
    total_revenue: f64,
}

/// Admin dashboard endpoint
async fn admin_dashboard(token_user: TokenUser) -> Result<Json<AdminDashboard>, Error> {
    debug!(
        user_id = token_user.id,
        username = %token_user.username,
        "Admin accessing dashboard"
    );
    
    // In a real application, you would fetch this data from the database
    let dashboard = AdminDashboard {
        total_users: 100,
        total_products: 500,
        total_orders: 1000,
        total_revenue: 50000.0,
    };
    
    // Format the response
    let response = response_formatter::format_ok(dashboard);
    
    Ok(Json(response.body.unwrap()))
}

#[derive(Debug, Deserialize)]
struct ListUsersQuery {
    #[serde(default = "default_page")]
    page: u64,
    #[serde(default = "default_limit")]
    limit: u32,
    role: Option<String>,
    search: Option<String>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u32 {
    20
}

#[derive(Debug, Serialize)]
struct UserListItem {
    id: i32,
    username: String,
    role: String,
    created_at: String,
}

/// List users endpoint
async fn list_users(
    token_user: TokenUser,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<Vec<UserListItem>>, Error> {
    debug!(
        user_id = token_user.id,
        username = %token_user.username,
        query = ?query,
        "Admin listing users"
    );
    
    // In a real application, you would fetch this data from the database
    // and apply the query parameters
    let users = vec![
        UserListItem {
            id: 1,
            username: "admin".to_string(),
            role: "admin".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
        },
        UserListItem {
            id: 2,
            username: "vendor1".to_string(),
            role: "vendor".to_string(),
            created_at: "2023-01-02T00:00:00Z".to_string(),
        },
        UserListItem {
            id: 3,
            username: "buyer1".to_string(),
            role: "buyer".to_string(),
            created_at: "2023-01-03T00:00:00Z".to_string(),
        },
    ];
    
    // Format the response
    let response = response_formatter::format_paginated_success(
        users,
        StatusCode::OK,
        3, // Total count
        0, // Offset
        query.limit,
    );
    
    Ok(Json(response.body.unwrap()))
}

#[derive(Debug, Serialize)]
struct SystemStatus {
    status: String,
    version: String,
    uptime: u64,
    memory_usage: u64,
    cpu_usage: f64,
    database_status: String,
}

/// System status endpoint
async fn system_status(token_user: TokenUser) -> Result<Json<SystemStatus>, Error> {
    debug!(
        user_id = token_user.id,
        username = %token_user.username,
        "Admin checking system status"
    );
    
    // In a real application, you would fetch this data from the system
    let status = SystemStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 3600, // 1 hour
        memory_usage: 1024 * 1024 * 100, // 100 MB
        cpu_usage: 5.0, // 5%
        database_status: "connected".to_string(),
    };
    
    // Format the response
    let response = response_formatter::format_ok(status);
    
    Ok(Json(response.body.unwrap()))
}
