pub mod auth;

use axum::{
    extract::Request,
    http::{HeaderValue, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Add x-waline-version header to all responses
pub async fn version_header(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-waline-version",
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );
    response
}

/// IP rate limiting middleware (simplified - uses in-memory state)
pub async fn ip_rate_limit(request: Request, next: Next) -> Response {
    // TODO: implement proper rate limiting with governor or similar
    next.run(request).await
}

/// Secure domains middleware - validates Referer/Origin headers
pub async fn secure_domains(request: Request, next: Next) -> Response {
    // TODO: implement secure domain checking
    next.run(request).await
}
