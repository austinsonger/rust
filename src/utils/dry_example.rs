use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

// Example of DRY principle - reusable data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceId {
    pub id: u32,
}

// Example of DRY principle - reusable response type
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub status_code: u16,
}

// Example of DRY principle - reusable response builder
impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            status_code: 200,
        }
    }

    pub fn error(message: impl Into<String>, status_code: u16) -> ApiResponse<T> {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
            status_code,
        }
    }

    pub fn not_found(resource: &str) -> ApiResponse<T> {
        Self::error(format!("{} not found", resource), 404)
    }

    pub fn bad_request(message: impl Into<String>) -> ApiResponse<T> {
        Self::error(message, 400)
    }

    pub fn internal_error() -> ApiResponse<T> {
        Self::error("Internal server error", 500)
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let status = match self.status_code {
            400 => StatusCode::BAD_REQUEST,
            404 => StatusCode::NOT_FOUND,
            500 => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::OK,
        };

        (status, Json(self)).into_response()
    }
}

// Example of DRY principle - reusable handler function
pub async fn get_resource<T, F>(
    Path(id): Path<u32>,
    finder: F,
    resource_name: &'static str,
) -> impl IntoResponse
where
    T: Serialize,
    F: FnOnce(u32) -> Option<T>,
{
    debug!("Getting {} with ID: {}", resource_name, id);

    match finder(id) {
        Some(resource) => {
            info!("Found {} with ID: {}", resource_name, id);
            ApiResponse::success(resource)
        }
        None => {
            error!("{} with ID {} not found", resource_name, id);
            ApiResponse::not_found(resource_name)
        }
    }
}

// Example of DRY principle - reusable query parameters
#[derive(Debug, Deserialize, Clone)]
pub struct ListParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub filter: Option<String>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u32 {
    20
}

// Example of DRY principle - reusable list response
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u32,
    pub pages: u64,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>, total: u64, params: &ListParams) -> Self {
        let pages = (total as f64 / params.limit as f64).ceil() as u64;

        Self {
            items,
            total,
            page: params.page,
            limit: params.limit,
            pages,
        }
    }
}

// Example of DRY principle - reusable list handler
pub async fn list_resources<T, F>(
    Query(params): Query<ListParams>,
    lister: F,
    resource_name: &'static str,
) -> impl IntoResponse
where
    T: Serialize,
    F: FnOnce(&ListParams) -> (Vec<T>, u64),
{
    debug!("Listing {}s with params: {:?}", resource_name, params);

    let (items, total) = lister(&params);
    let items_len = items.len();
    let response = ListResponse::new(items, total, &params);

    info!("Listed {} {}s", items_len, resource_name);
    ApiResponse::success(response)
}

// Example of DRY principle - reusable create handler
pub async fn create_resource<T, U, F>(
    Json(data): Json<U>,
    creator: F,
    resource_name: &'static str,
) -> impl IntoResponse
where
    T: Serialize,
    U: std::fmt::Debug,
    F: FnOnce(U) -> Result<T, String>,
{
    debug!("Creating {} with data: {:?}", resource_name, data);

    match creator(data) {
        Ok(resource) => {
            info!("Created new {}", resource_name);
            (StatusCode::CREATED, Json(ApiResponse::success(resource)))
        }
        Err(err) => {
            error!("Failed to create {}: {}", resource_name, err);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<T>::bad_request(err)),
            )
        }
    }
}

// Example of DRY principle - reusable update handler
pub async fn update_resource<T, U, F>(
    Path(id): Path<u32>,
    Json(data): Json<U>,
    updater: F,
    resource_name: &'static str,
) -> impl IntoResponse
where
    T: Serialize,
    U: std::fmt::Debug,
    F: FnOnce(u32, U) -> Result<T, String>,
{
    debug!("Updating {} with ID: {} and data: {:?}", resource_name, id, data);

    match updater(id, data) {
        Ok(resource) => {
            info!("Updated {} with ID: {}", resource_name, id);
            Json(ApiResponse::success(resource))
        }
        Err(err) => {
            error!("Failed to update {} with ID {}: {}", resource_name, id, err);
            Json(ApiResponse::<T>::bad_request(err))
        }
    }
}

// Example of DRY principle - reusable delete handler
pub async fn delete_resource<F>(
    Path(id): Path<u32>,
    deleter: F,
    resource_name: &'static str,
) -> impl IntoResponse
where
    F: FnOnce(u32) -> Result<(), String>,
{
    debug!("Deleting {} with ID: {}", resource_name, id);

    match deleter(id) {
        Ok(()) => {
            info!("Deleted {} with ID: {}", resource_name, id);
            (StatusCode::NO_CONTENT, Json(ApiResponse::<()>::success(())))
        }
        Err(err) => {
            error!("Failed to delete {} with ID {}: {}", resource_name, id, err);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::bad_request(err)),
            )
        }
    }
}
