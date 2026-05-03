use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{response::Json as JsonResponse, services::db as svc, state::AppState, utils::jwt};

fn require_admin(headers: &HeaderMap, state: &AppState) -> Result<i64, ()> {
    let token = crate::utils::extract_token(headers).ok_or(())?;
    let uid = jwt::object_id_from_token(&token, &state.config.jwt_secret()).map_err(|_| ())?;
    Ok(uid)
}

async fn check_admin(headers: &HeaderMap, state: &AppState) -> bool {
    match require_admin(headers, state) {
        Ok(uid) => crate::services::comment::is_admin(&state.db, uid).await.unwrap_or(false),
        Err(_) => false,
    }
}

#[derive(Debug, Deserialize)]
pub struct TableQuery {
    pub table: Option<String>,
    #[serde(rename = "objectId")]
    pub object_id: Option<i64>,
}

// GET /api/db — export all data
pub async fn export_db(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    if !check_admin(&headers, &state).await {
        return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response();
    }
    match svc::export(&state.db).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(e) => e.into_response(),
    }
}

// POST /api/db?table=Comment — insert one record
pub async fn import_record(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(q): Query<TableQuery>,
    Json(mut body): Json<Value>,
) -> impl IntoResponse {
    if !check_admin(&headers, &state).await {
        return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response();
    }
    let table = q.table.as_deref().unwrap_or("");
    // Remove objectId before insert
    if let Some(obj) = body.as_object_mut() { obj.remove("objectId"); }

    match svc::insert_record(&state.db, table, &body).await {
        Ok(id) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": { "objectId": id } })).into_response(),
        Err(e) => e.into_response(),
    }
}

// PUT /api/db?table=Comment&objectId=1 — update one record
pub async fn update_record(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(q): Query<TableQuery>,
    Json(mut body): Json<Value>,
) -> impl IntoResponse {
    if !check_admin(&headers, &state).await {
        return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response();
    }
    let table = q.table.as_deref().unwrap_or("");
    let object_id = match q.object_id {
        Some(id) => id,
        None => return JsonResponse(json!({ "errno": 1000, "errmsg": "objectId required" })).into_response(),
    };
    if let Some(obj) = body.as_object_mut() {
        obj.remove("objectId");
        obj.remove("createdAt");
        obj.remove("updatedAt");
    }
    match svc::update_record(&state.db, table, object_id, &body).await {
        Ok(_) => JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response(),
        Err(e) => e.into_response(),
    }
}

// DELETE /api/db?table=Comment — clear table
pub async fn delete_db(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(q): Query<TableQuery>,
) -> impl IntoResponse {
    if !check_admin(&headers, &state).await {
        return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response();
    }
    let table = q.table.as_deref().unwrap_or("");
    match svc::clear_table(&state.db, table).await {
        Ok(_) => JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response(),
        Err(e) => e.into_response(),
    }
}
