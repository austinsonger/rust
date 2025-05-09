use std::net::SocketAddr;
use axum::{
    routing::get,
    Router,
    response::Html,
    http::header,
};
use tower_http::services::ServeDir;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Create a simple router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/login", get(login_handler))
        .route("/register", get(register_handler))
        .route("/products", get(products_handler))
        // Serve static files
        .nest_service("/static", ServeDir::new(PathBuf::from("static")));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

// Handler functions
async fn root_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn login_handler() -> Html<&'static str> {
    Html(include_str!("../static/login.html"))
}

async fn register_handler() -> Html<&'static str> {
    Html(include_str!("../static/register.html"))
}

async fn products_handler() -> Html<&'static str> {
    Html(include_str!("../static/products.html"))
}
