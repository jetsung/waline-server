use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use crate::HandlerResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VerificationQuery {
    pub token: Option<String>,
}

/// GET /api/verification - Verify email token
pub async fn verify_email(
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<VerificationQuery>,
) -> HandlerResult<Json<serde_json::Value>> {
    let token = query.token.as_deref().unwrap_or("");
    let claims = waline_auth::jwt::verify_token(token, &state.config.jwt_key)
        .map_err(|e| waline_common::error::WalineError::Auth(e))?;

    // Update user type from verify:TOKEN:EXPIRY to guest
    let update = waline_common::models::UserUpdate {
        user_type: Some("guest".to_string()),
        ..Default::default()
    };
    state.db.update_user(&claims.sub, &update).await?;

    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
