use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VerificationQuery {
    pub email: String,
    pub token: String,
}

pub async fn verify_email(
    State(state): State<AppState>,
    Query(q): Query<VerificationQuery>,
) -> impl IntoResponse {
    match crate::services::user::verify_email(&state.db, &q.email, &q.token).await {
        Ok(_) => Redirect::temporary("/ui/login").into_response(),
        Err(_) => (StatusCode::BAD_REQUEST, "Token expired or invalid").into_response(),
    }
}
