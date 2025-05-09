use axum::{
    http::header,
    middleware,
    Router,
};
use std::path::PathBuf;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    services::ServeDir,
    trace,
};
use tracing::info;

use crate::{
    constants::http::MAX_REQUEST_BODY_SIZE,
    database,
    logger,
    middleware::request_id::request_id_middleware,
    models,
    routes,
    settings::SETTINGS,
};

pub async fn create_app() -> Router {
    // Set up logging
    logger::setup();

    info!("Initializing application...");

    // Run database migrations
    if let Err(err) = database::run_migrations() {
        tracing::error!(error = %err, "Failed to run database migrations");
        panic!("Failed to run database migrations: {}", err);
    }

    // Check database availability
    if !database::check_database_availability().await {
        tracing::error!("Database is not available, but continuing startup");
    }

    // Register custom types for Diesel
    models::register_custom_types();

    // Create the router with all routes
    let app = Router::new()
        .merge(routes::frontend::create_route())
        .merge(routes::status::create_route())
        .merge(routes::user::create_route())
        .merge(routes::product::create_route())
        // Use our DRY implementation for products as an alternative
        // .merge(routes::product_dry::create_route())
        .merge(routes::order::create_route())
        .merge(routes::message::create_route())
        .merge(routes::payment::create_route())
        .merge(routes::vendor::create_route())
        .merge(routes::admin::create_route())
        // Serve static files
        .nest_service("/static", ServeDir::new(PathBuf::from("static")))
        // Add request ID middleware
        .layer(middleware::from_fn(request_id_middleware))
        // Limit request body size
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_SIZE))
        // High level logging of requests and responses
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        // Mark sensitive headers so they don't show in logs
        .layer(SetSensitiveHeadersLayer::new(vec![
            header::AUTHORIZATION,
            header::COOKIE,
            header::HeaderName::from_static("x-api-key"),
        ]))
        // Compress responses
        .layer(CompressionLayer::new())
        // Propagate request ID from requests to responses
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )));

    // Add CORS configuration based on environment
    let app = if SETTINGS.environment == "production" {
        // More restrictive CORS for production
        app.layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(vec![
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                ])
                .allow_headers(vec![
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::HeaderName::from_static("x-request-id"),
                ])
                .max_age(std::time::Duration::from_secs(86400)), // 24 hours
        )
    } else {
        // Permissive CORS for development
        app.layer(CorsLayer::permissive())
    };

    info!("Application initialized successfully");
    app
}
