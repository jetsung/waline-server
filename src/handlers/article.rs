use axum::{Json, extract::{Query, State}, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

use crate::{response::Json as JsonResponse, services::article, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ArticleGetQuery {
    /// path can be repeated: ?path=/a&path=/b  or comma-joined
    pub path: Option<String>,
    #[serde(rename = "type")]
    pub types: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArticlePostBody {
    pub path: Option<String>,
    #[serde(rename = "type", default = "default_time")]
    pub field: String,
    #[serde(default = "default_inc")]
    pub action: String,
}

fn default_time() -> String { "time".to_string() }
fn default_inc() -> String { "inc".to_string() }

pub async fn get_article(
    State(state): State<AppState>,
    Query(q): Query<ArticleGetQuery>,
) -> impl IntoResponse {
    let paths: Vec<String> = q.path.as_deref()
        .map(|p| p.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let types: Vec<String> = q.types.as_deref()
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_else(|| vec!["time".to_string()]);

    match article::get_counters(&state.db, &paths, &types).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn post_article(
    State(state): State<AppState>,
    Json(body): Json<ArticlePostBody>,
) -> impl IntoResponse {
    let path = body.path.as_deref().unwrap_or("");
    match article::update_counter(&state.db, path, &body.field, &body.action).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(e) => e.into_response(),
    }
}
