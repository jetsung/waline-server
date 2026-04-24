use axum::{extract::State, Json};
use serde::Deserialize;
use crate::{state::AppState, HandlerResult, AppError};
use waline_common::error::WalineError;

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// PUT /api/user/password - Send password reset email
pub async fn reset_password(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<PasswordResetRequest>,
) -> HandlerResult<Json<serde_json::Value>> {
    let user = state.db.get_user_by_email(&body.email).await?;
    if let Some(user) = user {
        // Create a short-lived JWT token for password reset
        let claims = waline_auth::jwt::TokenClaims::new(&user.object_id, &user.user_type, 1); // 1 hour
        let token = waline_auth::jwt::create_token(&claims, &state.config.jwt_key)
            .map_err(|e| AppError(WalineError::Internal(e)))?;

        // TODO: Send password reset email with the token
        // For now, just return success (email sending not yet implemented)
        if state.config.has_smtp() {
            tracing::info!("Password reset email would be sent to: {}", body.email);
        }
    }
    // Always return success to prevent email enumeration
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
