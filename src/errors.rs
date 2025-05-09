use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::result::Error as DieselError;
use serde_json::json;
use std::error::Error as StdError;
use tokio::task::JoinError;
use tracing::{error, warn};

/// Application error types
/// These errors are used throughout the application and are converted to HTTP responses
#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub enum Error {
    #[error("Database error: {message}")]
    DatabaseError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        code: Option<String>,
    },

    #[error("Error parsing ID: {0}")]
    ParseIDError(String),

    #[error("{0}")]
    Authenticate(#[from] AuthenticateError),

    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("Task execution error: {0}")]
    RunSyncTask(#[from] JoinError),

    #[error("Password hashing error: {0}")]
    HashPassword(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("PGP error: {0}")]
    PGPError(String),

    #[error("Cryptocurrency error: {message}")]
    CryptoError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    #[error("Tor network error: {message}")]
    TorError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {message}")]
    InternalServerError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        request_id: Option<String>,
    },
}

impl Error {
    fn get_codes(&self) -> (StatusCode, u16) {
        match self {
            // 4XX Errors
            Error::ParseIDError(_) => (StatusCode::BAD_REQUEST, 40001),
            Error::BadRequest(_) => (StatusCode::BAD_REQUEST, 40002),
            Error::NotFound(_) => (StatusCode::NOT_FOUND, 40003),
            Error::Authenticate(AuthenticateError::WrongCredentials) => {
                (StatusCode::UNAUTHORIZED, 40004)
            }
            Error::Authenticate(AuthenticateError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, 40005)
            }
            Error::Authenticate(AuthenticateError::Locked) => (StatusCode::LOCKED, 40006),
            Error::ValidationError(_) => (StatusCode::BAD_REQUEST, 40007),
            Error::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, 40008),

            // 5XX Errors
            Error::Authenticate(AuthenticateError::TokenCreation) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 50001)
            }
            Error::DatabaseError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, 50002),
            Error::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50003),
            Error::HashPassword(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50004),
            Error::EncryptionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50005),
            Error::DecryptionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50006),
            Error::PGPError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50007),
            Error::CryptoError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, 50008),
            Error::TorError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, 50009),
            Error::InternalServerError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, 50010),
        }
    }

    /// Create a new bad request error
    pub fn bad_request() -> Self {
        Error::BadRequest(BadRequest {})
    }

    /// Create a new not found error
    pub fn not_found() -> Self {
        Error::NotFound(NotFound {})
    }

    /// Create a new validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Error::ValidationError(message.into())
    }

    /// Create a new database error
    pub fn database_error(
        message: impl Into<String>,
        source: Option<Box<dyn StdError + Send + Sync>>,
        code: Option<String>
    ) -> Self {
        Error::DatabaseError {
            message: message.into(),
            source,
            code,
        }
    }

    /// Create a new internal server error
    pub fn internal_error(
        message: impl Into<String>,
        source: Option<Box<dyn StdError + Send + Sync>>,
        request_id: Option<String>
    ) -> Self {
        Error::InternalServerError {
            message: message.into(),
            source,
            request_id,
        }
    }
}

impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Self {
        Error::internal_error(
            err.to_string(),
            Some(err),
            None
        )
    }
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::NotFound => Error::not_found(),
            _ => Error::database_error(
                err.to_string(),
                Some(Box::new(err)),
                None
            ),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_codes();
        let message = self.to_string();

        // Log the error with appropriate level based on status code
        match status_code.as_u16() {
            500..=599 => error!(
                error_code = code,
                status_code = status_code.as_u16(),
                error_message = %message,
                "Server error occurred"
            ),
            400..=499 => warn!(
                error_code = code,
                status_code = status_code.as_u16(),
                error_message = %message,
                "Client error occurred"
            ),
            _ => {}
        }

        let body = Json(json!({
            "code": code,
            "message": message,
            "status": status_code.as_u16(),
            "error": true
        }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub enum AuthenticateError {
    #[error("Wrong authentication credentials")]
    WrongCredentials {
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        #[default]
        message: String = "Wrong authentication credentials".to_string(),
    },

    #[error("Failed to create authentication token")]
    TokenCreation {
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        #[default]
        message: String = "Failed to create authentication token".to_string(),
    },

    #[error("Invalid authentication credentials")]
    InvalidToken {
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        #[default]
        message: String = "Invalid authentication credentials".to_string(),
    },

    #[error("User is locked")]
    Locked {
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        #[default]
        message: String = "User is locked".to_string(),
        locked_at: Option<chrono::DateTime<chrono::Utc>>,
    },

    #[error("Two-factor authentication required")]
    TwoFactorRequired {
        #[default]
        message: String = "Two-factor authentication required".to_string(),
    },

    #[error("Two-factor authentication failed")]
    TwoFactorFailed {
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        #[default]
        message: String = "Two-factor authentication failed".to_string(),
    },

    #[error("PGP verification failed")]
    PGPVerificationFailed {
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
        #[default]
        message: String = "PGP verification failed".to_string(),
    },
}

// Simplified versions for backward compatibility
impl AuthenticateError {
    pub fn wrong_credentials() -> Self {
        AuthenticateError::WrongCredentials {
            source: None,
            message: "Wrong authentication credentials".to_string(),
        }
    }

    pub fn token_creation(source: Option<Box<dyn StdError + Send + Sync>>) -> Self {
        AuthenticateError::TokenCreation {
            source,
            message: "Failed to create authentication token".to_string(),
        }
    }

    pub fn invalid_token() -> Self {
        AuthenticateError::InvalidToken {
            source: None,
            message: "Invalid authentication credentials".to_string(),
        }
    }

    pub fn locked(locked_at: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        AuthenticateError::Locked {
            source: None,
            message: "User is locked".to_string(),
            locked_at,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Bad Request: {message}")]
pub struct BadRequest {
    #[default]
    pub message: String = "Bad Request".to_string(),
}

#[derive(thiserror::Error, Debug)]
#[error("Not found: {message}")]
pub struct NotFound {
    #[default]
    pub message: String = "Not found".to_string(),
}
