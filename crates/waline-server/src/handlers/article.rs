use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use crate::HandlerResult;
use waline_common::models::*;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub query_type: Option<String>,
}

/// GET /api/article - Get pageview counts
pub async fn get_article(
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<ArticleQuery>,
) -> HandlerResult<Json<serde_json::Value>> {
    let urls: Vec<String> = query
        .url
        .as_ref()
        .map(|u| u.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    let counters = state.db.select_counters(&urls).await?;
    let data: Vec<i64> = counters.iter().map(|c| c.time).collect();
    Ok(Json(serde_json::json!({ "data": data })))
}

/// POST /api/article - Increment/decrement pageview
pub async fn update_article(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> HandlerResult<Json<serde_json::Value>> {
    let url = body["url"].as_str().unwrap_or("");
    let action = body["action"].as_str().unwrap_or("incr");
    let increment = match action {
        "desc" => -1,
        _ => 1,
    };
    let counter = state.db.upsert_counter(url, increment).await?;
    Ok(Json(serde_json::json!({ "data": counter.time })))
}
