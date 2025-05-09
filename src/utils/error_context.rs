use std::error::Error as StdError;
use std::fmt::Debug;

use crate::errors::{AuthenticateError, Error};

/// A builder for creating error contexts
#[derive(Debug, Default)]
pub struct ErrorContextBuilder {
    message: Option<String>,
    source: Option<Box<dyn StdError + Send + Sync>>,
    request_id: Option<String>,
    code: Option<String>,
    user_id: Option<i32>,
    resource_id: Option<String>,
    resource_type: Option<String>,
    operation: Option<String>,
}

impl ErrorContextBuilder {
    /// Create a new error context builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the error message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
    
    /// Set the error source
    pub fn source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }
    
    /// Set the request ID
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
    
    /// Set the error code
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
    
    /// Set the user ID
    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// Set the resource ID
    pub fn resource_id<T: Debug>(mut self, resource_id: T) -> Self {
        self.resource_id = Some(format!("{:?}", resource_id));
        self
    }
    
    /// Set the resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }
    
    /// Set the operation
    pub fn operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
    
    /// Build a database error
    pub fn build_database_error(self) -> Error {
        let message = self.message.unwrap_or_else(|| {
            let resource_type = self.resource_type.unwrap_or_else(|| "resource".to_string());
            let operation = self.operation.unwrap_or_else(|| "operation".to_string());
            
            if let Some(resource_id) = &self.resource_id {
                format!("Database error during {} on {} with ID {}", operation, resource_type, resource_id)
            } else {
                format!("Database error during {} on {}", operation, resource_type)
            }
        });
        
        Error::database_error(message, self.source, self.code)
    }
    
    /// Build a validation error
    pub fn build_validation_error(self) -> Error {
        let message = self.message.unwrap_or_else(|| "Validation error".to_string());
        Error::validation_error(message)
    }
    
    /// Build a not found error
    pub fn build_not_found_error(self) -> Error {
        if let Some(message) = self.message {
            Error::NotFound(crate::errors::NotFound { message })
        } else {
            let resource_type = self.resource_type.unwrap_or_else(|| "Resource".to_string());
            
            if let Some(resource_id) = &self.resource_id {
                Error::NotFound(crate::errors::NotFound { 
                    message: format!("{} with ID {} not found", resource_type, resource_id) 
                })
            } else {
                Error::not_found()
            }
        }
    }
    
    /// Build an internal server error
    pub fn build_internal_error(self) -> Error {
        let message = self.message.unwrap_or_else(|| "Internal server error".to_string());
        Error::internal_error(message, self.source, self.request_id)
    }
    
    /// Build an authentication error for wrong credentials
    pub fn build_wrong_credentials_error(self) -> Error {
        let source = self.source;
        let message = self.message.unwrap_or_else(|| "Wrong authentication credentials".to_string());
        
        Error::Authenticate(AuthenticateError::WrongCredentials {
            source,
            message,
        })
    }
    
    /// Build an authentication error for invalid token
    pub fn build_invalid_token_error(self) -> Error {
        let source = self.source;
        let message = self.message.unwrap_or_else(|| "Invalid authentication credentials".to_string());
        
        Error::Authenticate(AuthenticateError::InvalidToken {
            source,
            message,
        })
    }
    
    /// Build an authentication error for token creation
    pub fn build_token_creation_error(self) -> Error {
        let source = self.source;
        let message = self.message.unwrap_or_else(|| "Failed to create authentication token".to_string());
        
        Error::Authenticate(AuthenticateError::TokenCreation {
            source,
            message,
        })
    }
    
    /// Build an authentication error for locked account
    pub fn build_locked_account_error(self, locked_at: Option<chrono::DateTime<chrono::Utc>>) -> Error {
        let source = self.source;
        let message = self.message.unwrap_or_else(|| "User is locked".to_string());
        
        Error::Authenticate(AuthenticateError::Locked {
            source,
            message,
            locked_at,
        })
    }
}
