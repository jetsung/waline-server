use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

/// CORS middleware — permissive (mirrors waline-mini behavior)
pub async fn cors(req: Request, next: Next) -> Response {
    if req.method() == Method::OPTIONS {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET,POST,PUT,DELETE,OPTIONS")
            .header(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "Content-Type,Authorization,X-Requested-With",
            )
            .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
            .body(Body::empty())
            .unwrap();
    }

    let mut resp = next.run(req).await;
    resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    resp
}

/// Secure-domains middleware: reject requests from unlisted origins when
/// SECURE_DOMAINS is configured.
pub async fn secure_domains(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if state.config.secure_domains.is_empty() {
        return next.run(req).await;
    }

    let origin = req
        .headers()
        .get(header::ORIGIN)
        .or_else(|| req.headers().get(header::REFERER))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let allowed = state.config.secure_domains.iter().any(|d| origin.contains(d.as_str()));

    if !allowed && !origin.is_empty() {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden"))
            .unwrap();
    }

    next.run(req).await
}
