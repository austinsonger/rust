use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{info, Span};
use tracing_subscriber::registry::LookupSpan;

use crate::{
    constants::http::REQUEST_ID_HEADER,
    logger::generate_request_id,
};

/// Middleware to add a request ID to each request
/// If the request already has a request ID header, it will be used
/// Otherwise, a new request ID will be generated
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Get or generate a request ID
    let request_id = if let Some(request_id) = request.headers().get(REQUEST_ID_HEADER) {
        request_id.to_str().unwrap_or_default().to_string()
    } else {
        generate_request_id()
    };

    // Add the request ID to the request headers
    request.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("")),
    );

    // Create a span with the request ID
    let span = tracing::info_span!("request", request_id = %request_id);

    // Log the request
    info!(
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        "Request received"
    );

    // Process the request within the span
    let response = {
        let _guard = span.enter();
        next.run(request).await
    };

    // Add the request ID to the response headers
    let mut response_with_id = response;
    response_with_id.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("")),
    );

    response_with_id
}

/// Extension trait to get the request ID from a span
pub trait RequestIdExt {
    fn request_id(&self) -> Option<String>;
}

impl<S> RequestIdExt for Span
where
    S: for<'a> LookupSpan<'a>,
{
    fn request_id(&self) -> Option<String> {
        self.with_subscriber(|subscriber| {
            let span = subscriber.downcast_ref::<S>()?;
            let extensions = span.extensions();
            extensions.get::<Arc<String>>().map(|s| (**s).clone())
        })
    }
}
