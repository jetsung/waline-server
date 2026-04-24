use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use waline_common::models::User;
use crate::state::AppState;

/// Extract user info from JWT token in request headers or query params.
/// Populates request extensions with the authenticated user (if any).
pub async fn extract_user(
    State(state): State<std::sync::Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(user) = extract_user_from_request(&request, &state.config.jwt_key) {
        request.extensions_mut().insert(user);
    }
    next.run(request).await
}

/// Extract user from Authorization: Bearer token or state query parameter
fn extract_user_from_request(request: &Request, jwt_key: &str) -> Option<User> {
    let token = extract_token(request)?;
    let claims = waline_auth::jwt::verify_token(&token, jwt_key).ok()?;

    Some(User {
        object_id: claims.sub.clone(),
        display_name: None,
        email: None,
        password: None,
        user_type: claims.r#type,
        url: None,
        avatar: None,
        label: None,
        github: None,
        twitter: None,
        facebook: None,
        google: None,
        weibo: None,
        qq: None,
        oidc: None,
        two_fa: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        level: None,
    })
}

/// Extract JWT token from request headers or query string
fn extract_token(request: &Request) -> Option<String> {
    // Try Authorization: Bearer <token> header
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
            // Some clients send just the token without Bearer prefix
            if !auth_str.is_empty() && !auth_str.contains(' ') {
                return Some(auth_str.to_string());
            }
        }
    }

    // Try state query parameter (used by some Waline clients)
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix("state=") {
                let token = urlencoding::decode(value).ok()?;
                return Some(token.into_owned());
            }
        }
    }

    None
}

/// Helper to get the authenticated user from request extensions
pub fn get_user_from_extensions(extensions: &axum::http::Extensions) -> Option<&User> {
    extensions.get::<User>()
}
