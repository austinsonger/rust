use axum::http::header;
use axum::Router;
use std::path::PathBuf;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveHeadersLayer, services::ServeDir, trace,
};

use crate::database;
use crate::logger;
use crate::models;
use crate::routes;

pub async fn create_app() -> Router {
    logger::setup();

    // Run database migrations
    database::run_migrations().expect("Failed to run database migrations");

    // Register custom types for Diesel
    models::register_custom_types();

    // Create the router with all routes
    Router::new()
        .merge(routes::frontend::create_route())
        .merge(routes::status::create_route())
        .merge(routes::user::create_route())
        .merge(routes::product::create_route())
        .merge(routes::order::create_route())
        .merge(routes::message::create_route())
        .merge(routes::payment::create_route())
        .merge(routes::vendor::create_route())
        // Serve static files
        .nest_service("/static", ServeDir::new(PathBuf::from("static")))
        // High level logging of requests and responses
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        // Mark the `Authorization` request header as sensitive so it doesn't
        // show in logs.
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        // Compress responses
        .layer(CompressionLayer::new())
        // Propagate `X-Request-Id`s from requests to responses
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )))
        // CORS configuration. This should probably be more restrictive in
        // production.
        .layer(CorsLayer::permissive())
}
