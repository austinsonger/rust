use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tracing::{debug, error, info};

use crate::{
    database::get_connection,
    errors::Error,
    utils::{
        authenticate_request::TokenUser,
        custom_response::{CustomResponse, CustomResponseBuilder, ResponsePagination},
    },
};

/// Generic function to handle creating a new resource
/// 
/// # Type Parameters
/// * `T` - The type of the resource to create
/// * `U` - The type of the request body
/// * `V` - The type of the response body
/// 
/// # Arguments
/// * `body` - The request body
/// * `creator_fn` - A function that creates the resource
/// * `resource_name` - The name of the resource for logging
/// 
/// # Returns
/// * `Result<CustomResponse<V>, Error>` - The created resource or an error
pub async fn create_resource<T, U, V>(
    body: Json<U>,
    creator_fn: impl FnOnce(U) -> Result<T, Error>,
    resource_name: &str,
) -> Result<CustomResponse<V>, Error>
where
    T: Into<V>,
    U: DeserializeOwned + Debug,
    V: Serialize,
{
    debug!("Creating new {}: {:?}", resource_name, body);
    
    // Create the resource
    let resource = creator_fn(body.0)?;
    
    // Convert to response type
    let response_body = resource.into();
    
    // Build the response
    let res = CustomResponseBuilder::new()
        .body(response_body)
        .status_code(StatusCode::CREATED)
        .build();
    
    info!("Created new {}", resource_name);
    Ok(res)
}

/// Generic function to handle getting a resource by ID
/// 
/// # Type Parameters
/// * `T` - The type of the resource to get
/// * `V` - The type of the response body
/// * `ID` - The type of the ID
/// 
/// # Arguments
/// * `id` - The ID of the resource
/// * `finder_fn` - A function that finds the resource
/// * `resource_name` - The name of the resource for logging
/// 
/// # Returns
/// * `Result<CustomResponse<V>, Error>` - The resource or an error
pub async fn get_resource_by_id<T, V, ID>(
    id: Path<ID>,
    finder_fn: impl FnOnce(ID) -> Result<T, Error>,
    resource_name: &str,
) -> Result<CustomResponse<V>, Error>
where
    T: Into<V>,
    V: Serialize,
    ID: Debug + Send,
{
    debug!("Getting {} with ID: {:?}", resource_name, id);
    
    // Find the resource
    let resource = finder_fn(id.0)?;
    
    // Convert to response type
    let response_body = resource.into();
    
    // Build the response
    let res = CustomResponseBuilder::new()
        .body(response_body)
        .status_code(StatusCode::OK)
        .build();
    
    debug!("Returning {}", resource_name);
    Ok(res)
}

/// Generic function to handle listing resources
/// 
/// # Type Parameters
/// * `T` - The type of the resources to list
/// * `V` - The type of the response body items
/// * `Q` - The type of the query parameters
/// 
/// # Arguments
/// * `query` - The query parameters
/// * `lister_fn` - A function that lists the resources
/// * `resource_name` - The name of the resources for logging
/// 
/// # Returns
/// * `Result<CustomResponse<Vec<V>>, Error>` - The resources or an error
pub async fn list_resources<T, V, Q>(
    query: Query<Q>,
    lister_fn: impl FnOnce(Q) -> Result<(Vec<T>, u64), Error>,
    resource_name: &str,
) -> Result<CustomResponse<Vec<V>>, Error>
where
    T: Into<V>,
    V: Serialize,
    Q: DeserializeOwned + Debug + Clone,
{
    debug!("Listing {}s with query: {:?}", resource_name, query);
    
    // Get the resources
    let (resources, count) = lister_fn(query.0)?;
    
    // Convert to response type
    let response_body = resources.into_iter().map(Into::into).collect();
    
    // Build the response with pagination
    let res = CustomResponseBuilder::new()
        .body(response_body)
        .status_code(StatusCode::OK)
        .pagination(ResponsePagination {
            count,
            offset: 0, // This should come from query
            limit: 20, // This should come from query
        })
        .build();
    
    debug!("Returning {}s", resource_name);
    Ok(res)
}

/// Generic function to handle updating a resource
/// 
/// # Type Parameters
/// * `T` - The type of the resource to update
/// * `U` - The type of the request body
/// * `V` - The type of the response body
/// * `ID` - The type of the ID
/// 
/// # Arguments
/// * `id` - The ID of the resource
/// * `body` - The request body
/// * `updater_fn` - A function that updates the resource
/// * `resource_name` - The name of the resource for logging
/// 
/// # Returns
/// * `Result<CustomResponse<V>, Error>` - The updated resource or an error
pub async fn update_resource<T, U, V, ID>(
    id: Path<ID>,
    body: Json<U>,
    updater_fn: impl FnOnce(ID, U) -> Result<T, Error>,
    resource_name: &str,
) -> Result<CustomResponse<V>, Error>
where
    T: Into<V>,
    U: DeserializeOwned + Debug,
    V: Serialize,
    ID: Debug + Send,
{
    debug!("Updating {} with ID: {:?}, body: {:?}", resource_name, id, body);
    
    // Update the resource
    let resource = updater_fn(id.0, body.0)?;
    
    // Convert to response type
    let response_body = resource.into();
    
    // Build the response
    let res = CustomResponseBuilder::new()
        .body(response_body)
        .status_code(StatusCode::OK)
        .build();
    
    info!("Updated {} with ID: {:?}", resource_name, id);
    Ok(res)
}

/// Generic function to handle deleting a resource
/// 
/// # Type Parameters
/// * `ID` - The type of the ID
/// 
/// # Arguments
/// * `id` - The ID of the resource
/// * `deleter_fn` - A function that deletes the resource
/// * `resource_name` - The name of the resource for logging
/// 
/// # Returns
/// * `Result<StatusCode, Error>` - Success status code or an error
pub async fn delete_resource<ID>(
    id: Path<ID>,
    deleter_fn: impl FnOnce(ID) -> Result<(), Error>,
    resource_name: &str,
) -> Result<StatusCode, Error>
where
    ID: Debug + Send,
{
    debug!("Deleting {} with ID: {:?}", resource_name, id);
    
    // Delete the resource
    deleter_fn(id.0)?;
    
    info!("Deleted {} with ID: {:?}", resource_name, id);
    Ok(StatusCode::NO_CONTENT)
}
