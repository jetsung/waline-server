use axum::{extract::{Query, State}, Json, response::{Redirect, IntoResponse}};
use serde::Deserialize;
use crate::HandlerResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct OAuthQuery {
    #[serde(rename = "type")]
    pub oauth_type: Option<String>,
    pub code: Option<String>,
    pub state: Option<String>,
}

/// GET /api/oauth - OAuth login/link flow
pub async fn oauth(
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<OAuthQuery>,
) -> HandlerResult<axum::response::Response> {
    let oauth_type = query.oauth_type.as_deref().unwrap_or("");
    let code = query.code.as_deref().unwrap_or("");

    if code.is_empty() {
        // Redirect to OAuth provider
        let url = waline_auth::oauth::get_oauth_redirect_url(&state.config.oauth_url, oauth_type);
        Ok(Redirect::to(&url).into_response())
    } else {
        // Handle callback
        let user_info = waline_auth::oauth::fetch_oauth_user(&state.config.oauth_url, oauth_type, code)
            .await
            .map_err(|e| waline_common::error::WalineError::Auth(e))?;

        // Check if social ID is linked to existing user
        if let Some(existing_user) = state.db.get_user_by_social(oauth_type, &user_info.id).await? {
            // Login existing user
            let claims = waline_auth::jwt::TokenClaims::new_default(&existing_user.object_id, &existing_user.user_type);
            let token = waline_auth::jwt::create_token(&claims, &state.config.jwt_key)
                .map_err(|e| waline_common::error::WalineError::Internal(e))?;
            Ok(Json(serde_json::json!({
                "token": token,
                "objectId": existing_user.object_id,
                "display_name": existing_user.display_name,
                "type": existing_user.user_type,
            })).into_response())
        } else {
            // Create new user
            let new_user = waline_common::models::NewUser {
                display_name: user_info.name.unwrap_or_else(|| "Anonymous".to_string()),
                email: user_info.email.unwrap_or_default(),
                password: String::new(), // OAuth users don't have passwords
            };
            let user = state.db.add_user(&new_user).await?;
            let claims = waline_auth::jwt::TokenClaims::new_default(&user.object_id, &user.user_type);
            let token = waline_auth::jwt::create_token(&claims, &state.config.jwt_key)
                .map_err(|e| waline_common::error::WalineError::Internal(e))?;
            Ok(Json(serde_json::json!({
                "token": token,
                "objectId": user.object_id,
                "display_name": user.display_name,
                "type": user.user_type,
            })).into_response())
        }
    }
}
