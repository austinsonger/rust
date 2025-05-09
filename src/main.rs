mod app;
mod constants;
mod database;
mod errors;
mod logger;
mod middleware;
mod models;
mod routes;
mod schema;
mod settings;
mod templates;
mod utils;

use std::net::SocketAddr;
use tracing::{error, info};

use crate::settings::SETTINGS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the application
    let app = app::create_app().await;

    // Set up the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], SETTINGS.server.port));
    info!("Server running on http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run the server with graceful shutdown
    match axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(err) => {
            error!(error = %err, "Server error");
            Err(Box::new(err))
        }
    }
}

// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received terminate signal, starting graceful shutdown");
        },
    }
}
