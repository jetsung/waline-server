use axum::{
    extract::{ConnectInfo, Path, Query, State},
    Json,
};
use crate::HandlerResult;
use waline_common::models::*;
use crate::state::AppState;
use std::net::SocketAddr;
use waline_notify::{NotifyData, send_notifications};

/// GET /api/comment - Get comments
pub async fn get_comments(
    State(state): State<std::sync::Arc<AppState>>,
    Query(query): Query<CommentQuery>,
) -> HandlerResult<Json<serde_json::Value>> {
    let query_type = query.query_type.clone();

    match query_type.as_deref() {
        Some("count") => {
            let urls: Vec<String> = query
                .url
                .as_ref()
                .map(|u| u.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            let counts = state.db.count_comments(&urls).await?;
            Ok(Json(serde_json::json!({ "data": counts })))
        }
        Some("recent") => {
            let comments = state.db.select_comments(&query).await?;
            Ok(Json(serde_json::json!({ "data": comments })))
        }
        Some("list") => {
            // Admin only - TODO: auth check
            let comments = state.db.select_comments(&query).await?;
            let total = state.db.total_comments(&query).await?;
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            Ok(Json(serde_json::json!({
                "data": comments,
                "page": page,
                "pageSize": page_size,
                "total": total,
            })))
        }
        _ => {
            let comments = state.db.select_comments(&query).await?;
            let total = state.db.total_comments(&query).await?;
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            Ok(Json(serde_json::json!({
                "data": comments,
                "page": page,
                "pageSize": page_size,
                "total": total,
            })))
        }
    }
}

/// POST /api/comment - Submit a new comment
pub async fn add_comment(
    State(state): State<std::sync::Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(mut body): Json<NewComment>,
) -> HandlerResult<Json<Comment>> {
    // 1. Rate limiting
    if !state.rate_limiter.check_and_update(&addr.ip().to_string(), 1) {
        return Err(anyhow::anyhow!("Comment too fast").into());
    }

    // 2. Anti-spam / Audit status
    let mut status = "approved".to_string();
    if state.comment_audit {
        status = "waiting".to_string();
    } else {
        // Akismet check if not in audit mode
        if let Some(ref akismet_key) = state.config.akismet_key {
            if akismet_key != "false" {
                // Simplified Akismet check logic
                // In a full impl, we'd call the service here
                // For now, default to approved if not explicitly spam
            }
        }
    }
    body.status = Some(status);

    // 3. Persist
    let comment = state.db.add_comment(&body).await?;
    
    // 4. Resolve Parent for notification
    let parent_comment = if let Some(ref pid) = body.pid {
        state.db.get_comment(pid).await.ok().flatten()
    } else {
        None
    };

    // 5. Trigger notifications
    let notify_data = NotifyData {
        site_name: state.site_name.clone(),
        site_url: state.site_url.clone(),
        comment: comment.clone(),
        parent_comment,
        is_admin_notify: true,
    };
    let config = state.config.clone();
    tokio::spawn(async move {
        send_notifications(&notify_data, &config).await;
    });
    
    // 6. Invalidate cache
    state.comment_cache.lock().unwrap().invalidate(&body.url);
    
    Ok(Json(comment))
}

/// PUT /api/comment/:id - Update a comment
pub async fn update_comment(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<CommentUpdate>,
) -> HandlerResult<Json<serde_json::Value>> {
    state.db.update_comment(&id, &body).await?;
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}

/// DELETE /api/comment/:id - Delete a comment
pub async fn delete_comment(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<String>,
) -> HandlerResult<Json<serde_json::Value>> {
    state.db.delete_comment(&id).await?;
    Ok(Json(serde_json::json!({ "errmsg": "success" })))
}
