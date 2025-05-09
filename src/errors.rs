use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::result::Error as DieselError;
use serde_json::json;
use std::error::Error as StdError;
use tokio::task::JoinError;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Error parsing ID {0}")]
    ParseIDError(String),

    #[error("{0}")]
    Authenticate(#[from] AuthenticateError),

    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("{0}")]
    RunSyncTask(#[from] JoinError),

    #[error("Password hashing error: {0}")]
    HashPassword(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("PGP error: {0}")]
    PGPError(String),

    #[error("Cryptocurrency error: {0}")]
    CryptoError(String),

    #[error("Tor network error: {0}")]
    TorError(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
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

            // 5XX Errors
            Error::Authenticate(AuthenticateError::TokenCreation) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 50001)
            }
            Error::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50002),
            Error::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50003),
            Error::HashPassword(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50004),
            Error::EncryptionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50005),
            Error::DecryptionError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50006),
            Error::PGPError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50007),
            Error::CryptoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50008),
            Error::TorError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50009),
            Error::InternalServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50010),
        }
    }

    pub fn bad_request() -> Self {
        Error::BadRequest(BadRequest {})
    }

    pub fn not_found() -> Self {
        Error::NotFound(NotFound {})
    }
}

impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Self {
        Error::InternalServerError(err.to_string())
    }
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::NotFound => Error::not_found(),
            _ => Error::DatabaseError(err.to_string()),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_codes();
        let message = self.to_string();
        let body = Json(json!({ "code": code, "message": message }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum AuthenticateError {
    #[error("Wrong authentication credentials")]
    WrongCredentials,
    #[error("Failed to create authentication token")]
    TokenCreation,
    #[error("Invalid authentication credentials")]
    InvalidToken,
    #[error("User is locked")]
    Locked,
}

#[derive(thiserror::Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {}
