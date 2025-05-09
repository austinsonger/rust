mod utils;

use std::net::SocketAddr;
use std::path::PathBuf;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json, Html},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use tower_http::{
    trace::TraceLayer,
    compression::CompressionLayer,
    services::ServeDir,
};

use utils::dry_example::{
    ApiResponse as DryApiResponse, ListParams, create_resource, delete_resource, get_resource,
    list_resources, update_resource,
};

// Example of DRY principle - reusable data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

// Example of DRY principle - reusable response type
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

// Example of DRY principle - reusable response builder
impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    fn error(message: impl Into<String>) -> ApiResponse<T> {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
        }
    }
}

// Example of DRY principle - reusable handler functions
async fn get_user(Path(id): Path<u32>) -> impl IntoResponse {
    // In a real app, this would fetch from a database
    let user = User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    };

    Json(ApiResponse::success(user))
}

async fn get_product(Path(id): Path<u32>) -> impl IntoResponse {
    // In a real app, this would fetch from a database
    let product = Product {
        id,
        name: format!("Product {}", id),
        price: id as f64 * 10.0,
    };

    Json(ApiResponse::success(product))
}

// Example of DRY principle - reusable error handler
fn handle_error<E: std::fmt::Display>(err: E) -> (StatusCode, Json<ApiResponse<()>>) {
    error!("Error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::error(format!("Internal server error: {}", err))),
    )
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a router with our routes
    let app = Router::new()
        // Frontend routes
        .route("/", get(home))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/products", get(products))
        .route("/product/:id", get(product_detail))

        // Original API routes
        .route("/users/:id", get(get_user))
        .route("/products/:id", get(get_product))

        // DRY routes using our reusable handlers
        .route("/api/users/:id", get(|path| get_resource(path, find_user, "user")))
        .route("/api/users", get(|query| list_resources(query, list_users, "user")))
        .route("/api/users", post(|body| create_resource(body, create_user, "user")))
        .route("/api/users/:id", put(|path, body| update_resource(path, body, update_user, "user")))
        .route("/api/users/:id", delete(|path| delete_resource(path, delete_user, "user")))

        .route("/api/products/:id", get(|path| get_resource(path, find_product, "product")))
        .route("/api/products", get(|query| list_resources(query, list_products, "product")))
        .route("/api/products", post(|body| create_resource(body, create_product, "product")))
        .route("/api/products/:id", put(|path, body| update_resource(path, body, update_product, "product")))
        .route("/api/products/:id", delete(|path| delete_resource(path, delete_product, "product")))

        // Serve static files
        .nest_service("/static", ServeDir::new(PathBuf::from("static")))

        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Example implementations of the finder functions
fn find_user(id: u32) -> Option<User> {
    // In a real app, this would fetch from a database
    Some(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

fn list_users(params: &ListParams) -> (Vec<User>, u64) {
    // In a real app, this would fetch from a database with pagination
    let users = (1..=10).map(|i| User {
        id: i,
        name: format!("User {}", i),
        email: format!("user{}@example.com", i),
    }).collect();

    (users, 100) // 100 total users
}

fn create_user(user: User) -> Result<User, String> {
    // In a real app, this would insert into a database
    Ok(user)
}

fn update_user(id: u32, user: User) -> Result<User, String> {
    // In a real app, this would update in a database
    if id != user.id {
        return Err("ID in path does not match ID in body".to_string());
    }
    Ok(user)
}

fn delete_user(id: u32) -> Result<(), String> {
    // In a real app, this would delete from a database
    Ok(())
}

// Example implementations for products
fn find_product(id: u32) -> Option<Product> {
    // In a real app, this would fetch from a database
    Some(Product {
        id,
        name: format!("Product {}", id),
        price: id as f64 * 10.0,
    })
}

fn list_products(params: &ListParams) -> (Vec<Product>, u64) {
    // In a real app, this would fetch from a database with pagination
    let products = (1..=10).map(|i| Product {
        id: i,
        name: format!("Product {}", i),
        price: i as f64 * 10.0,
    }).collect();

    (products, 100) // 100 total products
}

fn create_product(product: Product) -> Result<Product, String> {
    // In a real app, this would insert into a database
    Ok(product)
}

fn update_product(id: u32, product: Product) -> Result<Product, String> {
    // In a real app, this would update in a database
    if id != product.id {
        return Err("ID in path does not match ID in body".to_string());
    }
    Ok(product)
}

fn delete_product(id: u32) -> Result<(), String> {
    // In a real app, this would delete from a database
    Ok(())
}

// Frontend route handlers
async fn home() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

async fn login() -> impl IntoResponse {
    Html(include_str!("../static/login.html"))
}

async fn register() -> impl IntoResponse {
    Html(include_str!("../static/register.html"))
}

#[derive(Deserialize)]
struct ProductsQuery {
    page: Option<i32>,
    category: Option<i32>,
    search: Option<String>,
    currency: Option<String>,
}

async fn products(_query: Query<ProductsQuery>) -> impl IntoResponse {
    Html(include_str!("../static/products.html"))
}

async fn product_detail(Path(_id): Path<i32>) -> impl IntoResponse {
    Html(include_str!("../static/product_detail.html"))
}
