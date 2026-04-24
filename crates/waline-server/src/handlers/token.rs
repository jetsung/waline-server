use axum::{extract::State, Json};
use serde::Deserialize;
use waline_common::error::WalineError;
use crate::HandlerResult;
use crate::AppError;
use waline_common::models::*;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub code: Option<String>, // 2FA code
}

/// GET /api/token - Get current user info
pub async fn get_token(
    State(state): State<std::sync::Arc<AppState>>,
) -> HandlerResult<Json<serde_json::Value>> {
    // TODO: extract user from JWT token
    Ok(Json(serde_json::json!({ "errmsg": "not implemented" })))
}

/// POST /api/token - Login
pub async fn login(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> HandlerResult<Json<serde_json::Value>> {
    let user = state
        .db
        .get_user_by_email(&body.email)
        .await?
        .ok_or_else(|| WalineError::Auth("User not found".to_string()))?;

    // Check if banned
    if user.user_type == "banned" {
        return Err(AppError(WalineError::Auth("User is banned".to_string())));
    }

    // Verify password
    let password_hash = user.password.as_deref().unwrap_or("");
    if !waline_auth::phpass::verify_password(&body.password, password_hash) {
        return Err(AppError(WalineError::Auth("Invalid password".to_string())));
    }

    // Check 2FA
    if let Some(ref two_fa_secret) = user.two_fa {
        let code = body.code.as_deref().ok_or_else(|| {
            WalineError::Auth("2FA code required".to_string())
        })?;
        if !waline_auth::totp::verify_totp(two_fa_secret, code) {
            return Err(AppError(WalineError::Auth("Invalid 2FA code".to_string())));
        }
    }

    // Create JWT token
    let claims = waline_auth::jwt::TokenClaims::new_default(&user.object_id, &user.user_type);
    let token = waline_auth::jwt::create_token(&claims, &state.config.jwt_key)
        .map_err(|e| WalineError::Internal(e))?;

    Ok(Json(serde_json::json!({
        "token": token,
        "objectId": user.object_id,
        "display_name": user.display_name,
        "email": user.email,
        "type": user.user_type,
        "avatar": user.avatar,
    })))
}

/// DELETE /api/token - Logout
pub async fn logout() -> HandlerResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
