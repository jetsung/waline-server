use axum::{extract::State, Json};
use serde::Deserialize;
use waline_common::error::WalineError;
use crate::{state::AppState, HandlerResult, AppError};

#[derive(Debug, Deserialize)]
pub struct TwoFaSetupRequest {
    pub code: Option<String>,
}

/// GET /api/token/2fa - Get 2FA setup info
pub async fn get_2fa_setup(
    State(state): State<std::sync::Arc<AppState>>,
) -> HandlerResult<Json<serde_json::Value>> {
    // TODO: extract user from JWT token
    // For now, return placeholder
    let setup = waline_auth::totp::generate_totp_secret("user@waline", "Waline")
        .map_err(|e| AppError(WalineError::Internal(e)))?;
    Ok(Json(serde_json::json!({
        "secret": setup.secret,
        "url": setup.qr_url,
    })))
}

/// POST /api/token/2fa - Enable 2FA
pub async fn enable_2fa(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<TwoFaSetupRequest>,
) -> HandlerResult<Json<serde_json::Value>> {
    let code = body.code.as_deref().ok_or_else(|| {
        AppError(WalineError::BadRequest("2FA code required".to_string()))
    })?;
    // TODO: verify code against the user's pending secret, then save to DB
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
