use axum::{extract::{State, Query, Path}, Json};
use serde::Deserialize;
use waline_common::error::WalineError;
use crate::HandlerResult;
use waline_common::models::*;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub code: Option<String>, // CAPTCHA code
}

/// GET /api/user - Get user list
pub async fn get_users(
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<UserQuery>,
) -> HandlerResult<Json<serde_json::Value>> {
    let users = state.db.select_users(&query).await?;
    let total = state.db.count_users(&query).await?;
    Ok(Json(serde_json::json!({
        "data": users,
        "total": total,
    })))
}

/// POST /api/user - Register
pub async fn register(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> HandlerResult<Json<serde_json::Value>> {
    // Check if email already exists
    if state.db.get_user_by_email(&body.email).await?.is_some() {
        return Err(crate::AppError(WalineError::BadRequest("Email already registered".to_string())));
    }

    // Hash password
    let password_hash = waline_auth::phpass::hash_password(&body.password)
        .map_err(|e| WalineError::Internal(e))?;

    // First user becomes administrator
    let total = state.db.count_users(&UserQuery::default()).await?;
    let user_type = if total == 0 { "administrator" } else { "guest" };

    let new_user = NewUser {
        display_name: body.display_name,
        email: body.email,
        password: password_hash,
    };

    let user = state.db.add_user(&new_user).await?;

    // Update user type
    if user_type == "administrator" {
        let update = UserUpdate {
            user_type: Some("administrator".to_string()),
            ..Default::default()
        };
        state.db.update_user(&user.object_id, &update).await?;
    }

    // Create JWT token
    let claims = waline_auth::jwt::TokenClaims::new_default(&user.object_id, user_type);
    let token = waline_auth::jwt::create_token(&claims, &state.config.jwt_key)
        .map_err(|e| WalineError::Internal(e))?;

    Ok(Json(serde_json::json!({
        "token": token,
        "objectId": user.object_id,
        "display_name": user.display_name,
        "email": user.email,
        "type": user_type,
    })))
}

/// PUT /api/user - Update user profile
pub async fn update_user(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<UserUpdate>,
) -> HandlerResult<Json<serde_json::Value>> {
    // TODO: get user ID from JWT token
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}

/// DELETE /api/user/:id - Delete/ban user (admin only)
pub async fn delete_user(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> HandlerResult<Json<serde_json::Value>> {
    state.db.delete_user(&id).await?;
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
