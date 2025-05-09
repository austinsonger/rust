use axum::http::StatusCode;
use serde::Serialize;
use tracing::debug;

use crate::{
    errors::Error,
    utils::custom_response::{CustomResponse, CustomResponseBuilder, ResponsePagination},
};

/// Format a successful response with a body
/// 
/// # Type Parameters
/// * `T` - The type of the response body
/// 
/// # Arguments
/// * `body` - The response body
/// * `status_code` - The HTTP status code
/// 
/// # Returns
/// * `CustomResponse<T>` - The formatted response
pub fn format_success<T>(body: T, status_code: StatusCode) -> CustomResponse<T>
where
    T: Serialize,
{
    debug!("Formatting success response with status code: {}", status_code);
    
    CustomResponseBuilder::new()
        .body(body)
        .status_code(status_code)
        .build()
}

/// Format a successful response with a body and pagination
/// 
/// # Type Parameters
/// * `T` - The type of the response body
/// 
/// # Arguments
/// * `body` - The response body
/// * `status_code` - The HTTP status code
/// * `count` - The total count of items
/// * `offset` - The offset of the first item
/// * `limit` - The maximum number of items
/// 
/// # Returns
/// * `CustomResponse<T>` - The formatted response
pub fn format_paginated_success<T>(
    body: T,
    status_code: StatusCode,
    count: u64,
    offset: u64,
    limit: u32,
) -> CustomResponse<T>
where
    T: Serialize,
{
    debug!(
        "Formatting paginated success response with status code: {}, count: {}, offset: {}, limit: {}",
        status_code, count, offset, limit
    );
    
    CustomResponseBuilder::new()
        .body(body)
        .status_code(status_code)
        .pagination(ResponsePagination {
            count,
            offset,
            limit,
        })
        .build()
}

/// Format a created response
/// 
/// # Type Parameters
/// * `T` - The type of the response body
/// 
/// # Arguments
/// * `body` - The response body
/// 
/// # Returns
/// * `CustomResponse<T>` - The formatted response
pub fn format_created<T>(body: T) -> CustomResponse<T>
where
    T: Serialize,
{
    format_success(body, StatusCode::CREATED)
}

/// Format an OK response
/// 
/// # Type Parameters
/// * `T` - The type of the response body
/// 
/// # Arguments
/// * `body` - The response body
/// 
/// # Returns
/// * `CustomResponse<T>` - The formatted response
pub fn format_ok<T>(body: T) -> CustomResponse<T>
where
    T: Serialize,
{
    format_success(body, StatusCode::OK)
}

/// Format a no content response
/// 
/// # Returns
/// * `StatusCode` - The no content status code
pub fn format_no_content() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Try to perform an operation and format the result
/// 
/// # Type Parameters
/// * `T` - The type of the result
/// * `U` - The type of the response body
/// 
/// # Arguments
/// * `operation` - The operation to perform
/// * `converter` - A function to convert the result to the response body
/// * `status_code` - The HTTP status code for success
/// 
/// # Returns
/// * `Result<CustomResponse<U>, Error>` - The formatted response or an error
pub async fn try_operation<T, U, F, C>(
    operation: F,
    converter: C,
    status_code: StatusCode,
) -> Result<CustomResponse<U>, Error>
where
    F: std::future::Future<Output = Result<T, Error>>,
    C: FnOnce(T) -> U,
    U: Serialize,
{
    let result = operation.await?;
    let response_body = converter(result);
    
    Ok(format_success(response_body, status_code))
}

/// Try to perform a database operation and format the result
/// 
/// # Type Parameters
/// * `T` - The type of the result
/// * `U` - The type of the response body
/// 
/// # Arguments
/// * `operation` - The operation to perform
/// * `converter` - A function to convert the result to the response body
/// * `status_code` - The HTTP status code for success
/// * `error_message` - The error message for database errors
/// 
/// # Returns
/// * `Result<CustomResponse<U>, Error>` - The formatted response or an error
pub fn try_db_operation<T, U, F, C>(
    operation: F,
    converter: C,
    status_code: StatusCode,
    error_message: &str,
) -> Result<CustomResponse<U>, Error>
where
    F: FnOnce() -> Result<T, diesel::result::Error>,
    C: FnOnce(T) -> U,
    U: Serialize,
{
    let result = operation().map_err(|err| {
        match err {
            diesel::result::Error::NotFound => Error::not_found(),
            _ => Error::database_error(
                error_message.to_string(),
                Some(Box::new(err)),
                None,
            ),
        }
    })?;
    
    let response_body = converter(result);
    
    Ok(format_success(response_body, status_code))
}
