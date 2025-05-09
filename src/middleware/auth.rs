use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{debug, error, warn};

use crate::{
    errors::{AuthenticateError, Error},
    utils::{
        authenticate_request::TokenUser,
        error_context::ErrorContextBuilder,
    },
};

/// Middleware to require authentication
pub async fn require_auth(
    request: Request,
    next: Next,
) -> Response {
    // Extract the token user from the request extensions
    match request.extensions().get::<TokenUser>() {
        Some(token_user) => {
            debug!(
                user_id = token_user.id,
                username = %token_user.username,
                role = %token_user.role,
                "User authenticated"
            );
            
            // Continue with the request
            next.run(request).await
        }
        None => {
            warn!("Authentication required but no token user found");
            
            // Return an unauthorized response
            Error::Authenticate(AuthenticateError::invalid_token()).into_response()
        }
    }
}

/// Middleware to require a specific role
pub fn require_role(role: &'static str) -> impl Fn(Request, Next) -> Response + Clone {
    move |request: Request, next: Next| async move {
        // Extract the token user from the request extensions
        match request.extensions().get::<TokenUser>() {
            Some(token_user) if token_user.role == role || token_user.role == "admin" => {
                debug!(
                    user_id = token_user.id,
                    username = %token_user.username,
                    role = %token_user.role,
                    required_role = %role,
                    "User has required role"
                );
                
                // Continue with the request
                next.run(request).await
            }
            Some(token_user) => {
                warn!(
                    user_id = token_user.id,
                    username = %token_user.username,
                    role = %token_user.role,
                    required_role = %role,
                    "User does not have required role"
                );
                
                // Return a forbidden response
                (
                    StatusCode::FORBIDDEN,
                    format!("Role '{}' required", role),
                )
                    .into_response()
            }
            None => {
                warn!("Authentication required but no token user found");
                
                // Return an unauthorized response
                Error::Authenticate(AuthenticateError::invalid_token()).into_response()
            }
        }
    }
}

/// Middleware to require admin role
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Response {
    require_role("admin")(request, next).await
}

/// Middleware to require vendor role
pub async fn require_vendor(
    request: Request,
    next: Next,
) -> Response {
    require_role("vendor")(request, next).await
}

/// Middleware to require buyer role
pub async fn require_buyer(
    request: Request,
    next: Next,
) -> Response {
    require_role("buyer")(request, next).await
}

/// Middleware to require moderator role
pub async fn require_moderator(
    request: Request,
    next: Next,
) -> Response {
    require_role("moderator")(request, next).await
}

/// Middleware to check if the user is the owner of a resource
/// This middleware expects the resource ID to be in the request extensions
/// with the key "resource_id" and the owner ID to be in the request extensions
/// with the key "owner_id"
pub async fn require_ownership(
    request: Request,
    next: Next,
) -> Response {
    // Extract the token user from the request extensions
    let token_user = match request.extensions().get::<TokenUser>() {
        Some(token_user) => token_user,
        None => {
            warn!("Authentication required but no token user found");
            
            // Return an unauthorized response
            return Error::Authenticate(AuthenticateError::invalid_token()).into_response();
        }
    };
    
    // If the user is an admin, they can access any resource
    if token_user.role == "admin" {
        debug!(
            user_id = token_user.id,
            username = %token_user.username,
            role = %token_user.role,
            "Admin user bypassing ownership check"
        );
        
        // Continue with the request
        return next.run(request).await;
    }
    
    // Extract the resource ID and owner ID from the request extensions
    let resource_id = match request.extensions().get::<i32>().cloned() {
        Some(id) => id,
        None => {
            error!("Resource ID not found in request extensions");
            
            // Return a server error response
            return ErrorContextBuilder::new()
                .message("Resource ID not found in request extensions")
                .operation("ownership check")
                .build_internal_error()
                .into_response();
        }
    };
    
    let owner_id = match request.extensions().get::<i32>().cloned() {
        Some(id) => id,
        None => {
            error!("Owner ID not found in request extensions");
            
            // Return a server error response
            return ErrorContextBuilder::new()
                .message("Owner ID not found in request extensions")
                .operation("ownership check")
                .build_internal_error()
                .into_response();
        }
    };
    
    // Check if the user is the owner
    if token_user.id != owner_id {
        warn!(
            user_id = token_user.id,
            username = %token_user.username,
            role = %token_user.role,
            resource_id = resource_id,
            owner_id = owner_id,
            "User is not the owner of the resource"
        );
        
        // Return a forbidden response
        return (
            StatusCode::FORBIDDEN,
            "You do not have permission to access this resource",
        )
            .into_response();
    }
    
    debug!(
        user_id = token_user.id,
        username = %token_user.username,
        role = %token_user.role,
        resource_id = resource_id,
        owner_id = owner_id,
        "User is the owner of the resource"
    );
    
    // Continue with the request
    next.run(request).await
}
