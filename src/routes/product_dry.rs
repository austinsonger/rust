use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post, patch, delete},
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    database::get_connection,
    errors::Error,
    models::product::{Category, NewCategory, NewProduct, Product, ProductWithDetails},
    schema::products::dsl::*,
    utils::{
        authenticate_request::TokenUser,
        db_operations,
        response_formatter,
        route_handlers,
        validation,
    },
};

pub fn create_route() -> Router {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route("/products/:id", get(get_product).patch(update_product).delete(delete_product))
        .route("/categories", get(list_categories).post(create_category))
}

// Query parameters for listing products
#[derive(Debug, Deserialize, Clone)]
pub struct ListProductsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub category_id: Option<i32>,
    pub vendor_id: Option<i32>,
    pub search: Option<String>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u32 {
    20
}

/// List products with optional filtering
async fn list_products(
    token_user: TokenUser,
    query: Query<ListProductsQuery>,
) -> Result<Json<Vec<Product>>, Error> {
    debug!("Listing products with query: {:?}", query);
    
    // Calculate offset from page and limit
    let offset = (query.page - 1) * query.limit as u64;
    
    // Build the query
    let mut query_builder = products.into_boxed();
    
    // Apply filters if provided
    if let Some(cat_id) = query.category_id {
        query_builder = query_builder.filter(category_id.eq(cat_id));
    }
    
    if let Some(v_id) = query.vendor_id {
        query_builder = query_builder.filter(vendor_id.eq(v_id));
    }
    
    if let Some(search_term) = &query.search {
        query_builder = query_builder.filter(title.like(format!("%{}%", search_term)));
    }
    
    // Get a database connection
    let mut conn = get_connection()?;
    
    // Execute the query with pagination
    let results = query_builder
        .offset(offset as i64)
        .limit(query.limit as i64)
        .load::<Product>(&mut conn)
        .map_err(|err| {
            Error::database_error(
                format!("Failed to list products: {}", err),
                Some(Box::new(err)),
                None,
            )
        })?;
    
    // Get the total count
    let total_count = products
        .count()
        .get_result::<i64>(&mut conn)
        .map_err(|err| {
            Error::database_error(
                format!("Failed to count products: {}", err),
                Some(Box::new(err)),
                None,
            )
        })? as u64;
    
    // Format the response with pagination
    let response = response_formatter::format_paginated_success(
        results,
        StatusCode::OK,
        total_count,
        offset,
        query.limit,
    );
    
    Ok(Json(response.body.unwrap_or_default()))
}

/// Create a new product
async fn create_product(
    token_user: TokenUser,
    Json(body): Json<NewProduct>,
) -> Result<Json<Product>, Error> {
    // Validate the product data
    validation::validate_non_empty_string(&body.title, "title")?;
    validation::validate_non_empty_string(&body.description, "description")?;
    
    // Only vendors can create products
    if !token_user.role.eq("vendor") {
        return Err(Error::validation_error("Only vendors can create products"));
    }
    
    // Create the product using our db_operations utility
    let product = db_operations::insert_record(
        &NewProduct {
            vendor_id: token_user.id,
            ..body
        },
        products,
        "product",
    )?;
    
    info!(
        product_id = product.id,
        vendor_id = product.vendor_id,
        "Product created successfully"
    );
    
    // Format the response
    let response = response_formatter::format_created(product);
    
    Ok(Json(response.body.unwrap()))
}

/// Get a product by ID
async fn get_product(Path(id): Path<i32>) -> Result<Json<ProductWithDetails>, Error> {
    // Use our route_handlers utility to handle getting a resource by ID
    let result = route_handlers::get_resource_by_id(
        Path(id),
        |id| {
            // This is our finder function
            db_operations::find_by_id(id, products, products::id, "product")
        },
        "product",
    ).await?;
    
    // For demonstration, we're converting Product to ProductWithDetails
    // In a real implementation, you would fetch the related data
    let product_with_details = ProductWithDetails {
        product: result.body.unwrap(),
        images: Vec::new(),
        reviews: Vec::new(),
        vendor: None,
    };
    
    Ok(Json(product_with_details))
}

/// Update a product
async fn update_product(
    token_user: TokenUser,
    Path(id): Path<i32>,
    Json(body): Json<NewProduct>,
) -> Result<Json<Product>, Error> {
    // First, check if the product exists and belongs to the user
    let product = db_operations::find_by_id(id, products, products::id, "product")?;
    
    // Verify ownership
    if product.vendor_id != token_user.id && token_user.role != "admin" {
        return Err(Error::validation_error("You can only update your own products"));
    }
    
    // Update the product
    let updated_product = db_operations::update_record(
        id,
        &body,
        products,
        products::id,
        "product",
    )?;
    
    info!(
        product_id = updated_product.id,
        vendor_id = updated_product.vendor_id,
        "Product updated successfully"
    );
    
    // Format the response
    let response = response_formatter::format_ok(updated_product);
    
    Ok(Json(response.body.unwrap()))
}

/// Delete a product
async fn delete_product(
    token_user: TokenUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, Error> {
    // First, check if the product exists and belongs to the user
    let product = db_operations::find_by_id(id, products, products::id, "product")?;
    
    // Verify ownership
    if product.vendor_id != token_user.id && token_user.role != "admin" {
        return Err(Error::validation_error("You can only delete your own products"));
    }
    
    // Delete the product
    db_operations::delete_record(
        id,
        products,
        products::id,
        "product",
    )?;
    
    info!(
        product_id = id,
        vendor_id = product.vendor_id,
        "Product deleted successfully"
    );
    
    // Return no content status
    Ok(StatusCode::NO_CONTENT)
}

/// List categories
async fn list_categories(Query(query): Query<ListProductsQuery>) -> Result<Json<Vec<Category>>, Error> {
    // Use our route_handlers utility to handle listing resources
    let result = route_handlers::list_resources(
        Query(query),
        |query| {
            // This is our lister function
            // In a real implementation, you would use the query parameters
            let mut conn = get_connection()?;
            let results = crate::schema::categories::dsl::categories
                .load::<Category>(&mut conn)
                .map_err(|err| {
                    Error::database_error(
                        format!("Failed to list categories: {}", err),
                        Some(Box::new(err)),
                        None,
                    )
                })?;
            
            let count = results.len() as u64;
            Ok((results, count))
        },
        "category",
    ).await?;
    
    Ok(Json(result.body.unwrap_or_default()))
}

/// Create a new category
async fn create_category(
    token_user: TokenUser,
    Json(body): Json<NewCategory>,
) -> Result<Json<Category>, Error> {
    // Only admins can create categories
    if token_user.role != "admin" {
        return Err(Error::validation_error("Only admins can create categories"));
    }
    
    // Validate the category data
    validation::validate_non_empty_string(&body.name, "name")?;
    
    // Create the category
    let category = db_operations::insert_record(
        &body,
        crate::schema::categories::dsl::categories,
        "category",
    )?;
    
    info!(
        category_id = category.id,
        category_name = %category.name,
        "Category created successfully"
    );
    
    // Format the response
    let response = response_formatter::format_created(category);
    
    Ok(Json(response.body.unwrap()))
}
