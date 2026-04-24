use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use crate::HandlerResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DbQuery {
    pub table: Option<String>,
}

/// GET /api/db - Export all data (admin only)
pub async fn export_db(
    State(state): State<std::sync::Arc<AppState>>,
) -> HandlerResult<Json<serde_json::Value>> {
    let data = state.db.export_all().await?;
    Ok(Json(data))
}

/// POST /api/db - Import data (admin only)
pub async fn import_db(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> HandlerResult<Json<serde_json::Value>> {
    state.db.import_data(&body).await?;
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}

/// PUT /api/db - Update records (admin only)
pub async fn update_db(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> HandlerResult<Json<serde_json::Value>> {
    let table = body["table"].as_str().unwrap_or("");
    state.db.update_records(table, &body).await?;
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}

/// DELETE /api/db - Clear table data (admin only)
pub async fn clear_db(
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<DbQuery>,
) -> HandlerResult<Json<serde_json::Value>> {
    let table = query.table.as_deref().unwrap_or("");
    state.db.clear_table(table).await?;
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
