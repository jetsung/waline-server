use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    error::AppError,
    models::comment,
    services::{comment as svc, notify::{self, NotifyContext}},
    state::AppState,
    utils::{extract_ip, extract_token, jwt, spam, ua},
};

// ── Query / Body types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetCommentQuery {
    pub path: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub query_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(rename = "pageSize", default = "default_page_size")]
    pub page_size: i64,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<String>,
    pub owner: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
    #[serde(default = "default_count")]
    pub count: i64,
    pub lang: Option<String>,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 10 }
fn default_count() -> i64 { 10 }

#[derive(Debug, Deserialize)]
pub struct PostCommentBody {
    pub comment: String,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub nick: Option<String>,
    pub ua: Option<String>,
    pub url: String,
    pub pid: Option<i64>,
    pub rid: Option<i64>,
    pub at: Option<String>,
    #[serde(rename = "recaptchaV3")]
    pub recaptcha_v3: Option<String>,
    pub turnstile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentBody {
    pub comment: Option<String>,
    pub link: Option<String>,
    pub mail: Option<String>,
    pub nick: Option<String>,
    pub ua: Option<String>,
    pub url: Option<String>,
    pub status: Option<String>,
    pub like: Option<bool>,
    pub sticky: Option<bool>,
}

// ── Auth helper ───────────────────────────────────────────────────────────────

fn auth(headers: &HeaderMap, secret: &str) -> Option<i64> {
    extract_token(headers)
        .as_deref()
        .and_then(|t| jwt::object_id_from_token(t, secret).ok())
}

// ── GET /api/comment ──────────────────────────────────────────────────────────

pub async fn get_comment(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(q): Query<GetCommentQuery>,
) -> impl IntoResponse {
    let user_id = auth(&headers, &state.config.jwt_secret());
    let is_admin = if let Some(id) = user_id {
        svc::is_admin(&state.db, id).await.unwrap_or(false)
    } else { false };

    match q.query_type.as_deref().unwrap_or("") {
        "count" => {
            let urls: Vec<String> = q.url.as_deref()
                .map(|u| u.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            match get_comment_count(&state, &urls, user_id).await {
                Ok(d) => Json(json!({ "errno": 0, "errmsg": "", "data": d })).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "recent" => {
            match get_recent(&state, q.count.min(50), is_admin, user_id).await {
                Ok(d) => Json(json!({ "errno": 0, "errmsg": "", "data": d })).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "list" => {
            if !is_admin {
                return Json(json!({ "errno": 403, "errmsg": "Forbidden" })).into_response();
            }
            match get_admin_list(&state, q.page, q.page_size.min(100), q.owner.as_deref(), q.status.as_deref(), q.keyword.as_deref(), user_id).await {
                Ok(d) => Json(json!({ "errno": 0, "errmsg": "", "data": d })).into_response(),
                Err(e) => e.into_response(),
            }
        }
        _ => {
            let path = match q.path.as_deref().or(q.url.as_deref()) {
                Some(p) => p.to_string(),
                None => return Json(json!({ "errno": 1000, "errmsg": "path required" })).into_response(),
            };
            match get_list(&state, &path, q.page, q.page_size.min(100), q.sort_by.as_deref().unwrap_or("insertedAt_desc"), is_admin, user_id).await {
                Ok(d) => Json(json!({ "errno": 0, "errmsg": "", "data": d })).into_response(),
                Err(e) => e.into_response(),
            }
        }
    }
}

async fn get_comment_count(state: &AppState, urls: &[String], _user_id: Option<i64>) -> Result<Value, AppError> {
    if urls.is_empty() {
        let count = comment::Entity::find()
            .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
            .count(&state.db)
            .await?;
        return Ok(json!(count as i64));
    }
    let mut counts = Vec::with_capacity(urls.len());
    for url in urls {
        let count = comment::Entity::find()
            .filter(comment::Column::Url.eq(url))
            .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
            .count(&state.db)
            .await?;
        counts.push(count as i64);
    }
    if counts.len() == 1 { Ok(json!(counts[0])) } else { Ok(json!(counts)) }
}

async fn get_recent(state: &AppState, count: i64, is_admin: bool, user_id: Option<i64>) -> Result<Value, AppError> {
    let mut query = comment::Entity::find();
    apply_status_filter(&mut query, is_admin, user_id);
    let comments = query
        .order_by_desc(comment::Column::InsertedAt)
        .limit(count as u64)
        .all(&state.db)
        .await?;
    build_comment_list(state, &comments, is_admin, user_id.is_some()).await
}

async fn get_admin_list(
    state: &AppState,
    page: i64,
    page_size: i64,
    owner: Option<&str>,
    status: Option<&str>,
    keyword: Option<&str>,
    user_id: Option<i64>,
) -> Result<Value, AppError> {
    let offset = (page - 1) * page_size;
    let mut query = comment::Entity::find();
    
    if owner == Some("mine") {
        if let Some(id) = user_id {
            query = query.filter(comment::Column::UserId.eq(id));
        }
    }
    if let Some(s) = status {
        if s == "approved" {
            query = query.filter(comment::Column::Status.is_not_in(["waiting", "spam"]));
        } else {
            query = query.filter(comment::Column::Status.eq(s));
        }
    }
    if let Some(k) = keyword {
        query = query.filter(comment::Column::Comment.contains(k));
    }

    let total = query.clone().count(&state.db).await? as i64;
    let spam_count = comment::Entity::find()
        .filter(comment::Column::Status.eq("spam"))
        .count(&state.db)
        .await? as i64;
    let waiting_count = comment::Entity::find()
        .filter(comment::Column::Status.eq("waiting"))
        .count(&state.db)
        .await? as i64;

    let comments = query
        .order_by_desc(comment::Column::InsertedAt)
        .offset(offset as u64)
        .limit(page_size as u64)
        .all(&state.db)
        .await?;

    let data = build_comment_list(state, &comments, true, true).await?;
    let total_pages = (total + page_size - 1) / page_size;
    Ok(json!({ "data": data, "page": page, "pageSize": page_size, "totalPages": total_pages, "spamCount": spam_count, "waitingCount": waiting_count }))
}

async fn get_list(
    state: &AppState,
    path: &str,
    page: i64,
    page_size: i64,
    sort_by: &str,
    is_admin: bool,
    user_id: Option<i64>,
) -> Result<Value, AppError> {
    let offset = (page - 1) * page_size;

    let mut count_query = comment::Entity::find()
        .filter(comment::Column::Url.eq(path));
    apply_status_filter(&mut count_query, is_admin, user_id);
    let total = count_query.clone().count(&state.db).await? as i64;

    let mut root_count_query = comment::Entity::find()
        .filter(comment::Column::Url.eq(path))
        .filter(comment::Column::Rid.is_null());
    apply_status_filter(&mut root_count_query, is_admin, user_id);
    let root_count = root_count_query.count(&state.db).await? as i64;

    let mut root_query = comment::Entity::find()
        .filter(comment::Column::Url.eq(path))
        .filter(comment::Column::Rid.is_null());
    apply_status_filter(&mut root_query, is_admin, user_id);

    match sort_by {
        "insertedAt_asc" => {
            root_query = root_query.order_by_asc(comment::Column::InsertedAt);
        }
        "like_desc" => {
            root_query = root_query.order_by_desc(comment::Column::LikeCount);
            root_query = root_query.order_by_desc(comment::Column::InsertedAt);
        }
        _ => {
            root_query = root_query.order_by_desc(comment::Column::Sticky);
            root_query = root_query.order_by_desc(comment::Column::InsertedAt);
        }
    }

    let roots = root_query
        .offset(offset as u64)
        .limit(page_size as u64)
        .all(&state.db)
        .await?;

    let total_pages = (root_count + page_size - 1) / page_size;
    let mut data = Vec::new();

    let levels: Option<Vec<i64>> = state.config.levels.as_deref()
        .map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect());

    for root in &roots {
        let root_user = if let Some(uid) = root.user_id {
            svc::get_user_by_id(&state.db, uid).await?
        } else {
            None
        };
        let root_level = if let Some(ref lvls) = levels {
            Some(compute_level(&state.db, root.user_id, root.mail.as_deref())
                .await
                .map(|c| svc::get_level(c, lvls))
                .unwrap_or(0))
        } else {
            None
        };
        let mut root_json = svc::format_comment_with_opts(
            root,
            root_user.as_ref(),
            &state.config,
            is_admin,
            user_id.is_some(),
            root_level,
        );

        let mut child_query = comment::Entity::find()
            .filter(comment::Column::Url.eq(path))
            .filter(comment::Column::Rid.eq(root.id));
        apply_status_filter(&mut child_query, is_admin, user_id);
        let children = child_query
            .order_by_asc(comment::Column::InsertedAt)
            .all(&state.db)
            .await?;

        let mut children_json = Vec::new();
        for child in &children {
            let child_user = if let Some(uid) = child.user_id {
                svc::get_user_by_id(&state.db, uid).await?
            } else {
                None
            };
            let child_level = if let Some(ref lvls) = levels {
                Some(compute_level(&state.db, child.user_id, child.mail.as_deref())
                    .await
                    .map(|c| svc::get_level(c, lvls))
                    .unwrap_or(0))
            } else {
                None
            };
            let mut cj = svc::format_comment_with_opts(
                child,
                child_user.as_ref(),
                &state.config,
                is_admin,
                user_id.is_some(),
                child_level,
            );

            // reply_user: find parent comment
            if let Some(pid) = child.pid {
                let parent = if pid == root.id {
                    Some((root.nick.clone(), root.link.clone(), root.mail.clone()))
                } else {
                    children.iter().find(|c| c.id == pid)
                        .map(|c| (c.nick.clone(), c.link.clone(), c.mail.clone()))
                };
                if let Some((nick, link, mail)) = parent {
                    let av = crate::utils::avatar::gravatar_url(
                        mail.as_deref().unwrap_or(""),
                        state.config.gravatar_str.as_deref(),
                    );
                    cj["reply_user"] = json!({ "nick": nick, "link": link, "avatar": av });
                }
            }
            children_json.push(cj);
        }
        root_json["children"] = json!(children_json);
        data.push(root_json);
    }

    Ok(json!({ "count": total, "data": data, "page": page, "pageSize": page_size, "totalPages": total_pages }))
}

// ── POST /api/comment ─────────────────────────────────────────────────────────

pub async fn post_comment(
    headers: HeaderMap,
    State(state): State<AppState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Json(body): Json<PostCommentBody>,
) -> impl IntoResponse {
    use crate::models::comment::ActiveModel;

    let client_ip = extract_ip(&headers, &addr.ip().to_string());
    let user_id = auth(&headers, &state.config.jwt_secret());
    let is_admin = if let Some(id) = user_id {
        svc::is_admin(&state.db, id).await.unwrap_or(false)
    } else {
        false
    };

    if user_id.is_none() && state.config.is_login_force() {
        return Json(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response();
    }

    if !is_admin {
        // Captcha
        if let (Some(token), Some(secret)) = (&body.recaptcha_v3, &state.config.recaptcha_v3_secret) {
            if !spam::verify_recaptcha(secret, token).await {
                return Json(json!({ "errno": 1000, "errmsg": "Captcha failed" })).into_response();
            }
        }
        if let (Some(token), Some(secret)) = (&body.turnstile, &state.config.turnstile_secret) {
            if !spam::verify_turnstile(secret, token).await {
                return Json(json!({ "errno": 1000, "errmsg": "Captcha failed" })).into_response();
            }
        }
        if state.config.disallow_ip_list.contains(&client_ip) {
            return Json(json!({ "errno": 403, "errmsg": "Forbidden" })).into_response();
        }
        if !state.rate_limiter.check(&client_ip) {
            return Json(json!({ "errno": 1000, "errmsg": "Comment too fast!" })).into_response();
        }
        // Duplicate
        let dup_count = comment::Entity::find()
            .filter(comment::Column::Url.eq(&body.url))
            .filter(comment::Column::Mail.eq(body.mail.as_deref().unwrap_or("")))
            .filter(comment::Column::Nick.eq(body.nick.as_deref().unwrap_or("")))
            .filter(comment::Column::Comment.eq(&body.comment))
            .count(&state.db)
            .await
            .unwrap_or(0);
        if dup_count > 0 {
            return Json(json!({ "errno": 1000, "errmsg": "Duplicate Content" })).into_response();
        }
    }

    let status = if is_admin {
        "approved".to_string()
    } else if state.config.comment_audit {
        "waiting".to_string()
    } else if spam::has_forbidden_word(&body.comment, &state.config.forbidden_words) {
        "spam".to_string()
    } else {
        let is_spam = if let Some(key) = &state.config.akismet_key {
            spam::akismet_check(
                key,
                state.config.site_url.as_deref().unwrap_or(""),
                &client_ip,
                body.ua.as_deref().unwrap_or(""),
                body.nick.as_deref().unwrap_or(""),
                body.mail.as_deref().unwrap_or(""),
                &body.comment,
            )
            .await
        } else {
            false
        };
        if is_spam { "spam".to_string() } else { "approved".to_string() }
    };

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let active = ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        user_id: Set(user_id),
        comment: Set(Some(body.comment.clone())),
        inserted_at: Set(Some(now.clone())),
        ip: Set(Some(client_ip.clone())),
        link: Set(body.link.clone()),
        mail: Set(Some(body.mail.clone().unwrap_or_default())),
        nick: Set(Some(body.nick.clone().unwrap_or_default())),
        pid: Set(body.pid),
        rid: Set(body.rid),
        status: Set(status.clone()),
        ua: Set(body.ua.clone()),
        url: Set(Some(body.url.clone())),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
        sticky: Set(None),
        like_count: Set(None),
    };

    let comment_model = match active.insert(&state.db).await {
        Ok(m) => m,
        Err(e) => return AppError::from(e).into_response(),
    };

    let user = if let Some(uid) = user_id {
        svc::get_user_by_id(&state.db, uid).await.ok().flatten()
    } else {
        None
    };
    let mut resp = svc::format_comment(&comment_model, user.as_ref(), &state.config, is_admin);
    let (browser, os) = ua::parse(body.ua.as_deref().unwrap_or(""));
    resp["browser"] = json!(browser);
    resp["os"] = json!(os);

    // Webhook
    if let Some(wh) = &state.config.webhook {
        let c = comment_model.clone();
        let u = wh.clone();
        tokio::spawn(async move { svc::send_webhook(&u, &c).await });
    }

    // Notify
    if status != "spam" {
        let parent = if let Some(pid) = body.pid {
            fetch_comment(&state, pid).await.ok().flatten()
        } else {
            None
        };
        let config = state.config.clone();
        let raw = body.comment.clone();
        let c = comment_model.clone();
        let p = parent.clone();
        tokio::spawn(async move {
            let ctx = NotifyContext {
                config: &config,
                comment: &c,
                comment_user: None,
                parent: p.as_ref(),
                parent_user: None,
                raw_comment: &raw,
            };
            notify::run(&ctx, false).await;
        });
    }

    Json(json!({ "errno": 0, "errmsg": "", "data": resp })).into_response()
}

// ── PUT /api/comment/:id ──────────────────────────────────────────────────────

pub async fn put_comment(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateCommentBody>,
) -> impl IntoResponse {
    use crate::models::comment::ActiveModel;

    // Like-only: no auth needed
    if body.like.is_some() && body.status.is_none() && body.comment.is_none() && body.sticky.is_none() {
        let inc_max = state.config.like_inc_max as i64;
        let inc: i64 = if body.like.unwrap() {
            if inc_max <= 1 { 1 } else { use rand::RngExt; rand::rng().random_range(1..=inc_max) }
        } else { -1 };

        if let Some(existing) = comment::Entity::find_by_id(id).one(&state.db).await.ok().flatten() {
            let new_like = (existing.like_count.unwrap_or(0) + inc).max(0);
            let mut active: ActiveModel = existing.into();
            active.like_count = Set(Some(new_like));
            active.updated_at = Set(Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
            let _ = active.update(&state.db).await;
        }

        return match fetch_comment(&state, id).await {
            Ok(Some(c)) => {
                let user = if let Some(uid) = c.user_id {
                    svc::get_user_by_id(&state.db, uid).await.ok().flatten()
                } else {
                    None
                };
                Json(json!({ "errno": 0, "errmsg": "", "data": svc::format_comment(&c, user.as_ref(), &state.config, false) })).into_response()
            }
            _ => AppError::NotFound.into_response(),
        };
    }

    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return Json(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    let is_admin = svc::is_admin(&state.db, user_id).await.unwrap_or(false);

    let existing = match comment::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(c)) => c,
        Ok(None) => return AppError::NotFound.into_response(),
        Err(e) => return AppError::from(e).into_response(),
    };

    if !is_admin && existing.user_id != Some(user_id) {
        return Json(json!({ "errno": 403, "errmsg": "Forbidden" })).into_response();
    }

    let mut active: ActiveModel = existing.clone().into();
    if let Some(s) = &body.status { active.status = Set(s.clone()); }
    if let Some(s) = &body.sticky { active.sticky = Set(Some(if *s { 1 } else { 0 })); }
    if let Some(s) = &body.comment { active.comment = Set(Some(s.clone())); }
    if let Some(s) = &body.nick { active.nick = Set(Some(s.clone())); }
    if let Some(s) = &body.mail { active.mail = Set(Some(s.clone())); }
    if let Some(s) = &body.link { active.link = Set(Some(s.clone())); }
    if let Some(s) = &body.url { active.url = Set(Some(s.clone())); }
    if let Some(s) = &body.ua { active.ua = Set(Some(s.clone())); }
    active.updated_at = Set(Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));

    let updated = match active.update(&state.db).await {
        Ok(u) => u,
        Err(e) => return AppError::from(e).into_response(),
    };

    // Notify on waiting→approved with parent
    if body.status.as_deref() == Some("approved") {
        if let Some(pid) = updated.pid {
            if let Ok(Some(parent)) = fetch_comment(&state, pid).await {
                let config = state.config.clone();
                let raw = updated.comment.clone().unwrap_or_default();
                let c = updated.clone();
                tokio::spawn(async move {
                    let ctx = NotifyContext {
                        config: &config,
                        comment: &c,
                        comment_user: None,
                        parent: Some(&parent),
                        parent_user: None,
                        raw_comment: &raw,
                    };
                    notify::run(&ctx, true).await;
                });
            }
        }
    }

    let user = if let Some(uid) = updated.user_id {
        svc::get_user_by_id(&state.db, uid).await.ok().flatten()
    } else {
        None
    };
    Json(json!({ "errno": 0, "errmsg": "", "data": svc::format_comment(&updated, user.as_ref(), &state.config, is_admin) })).into_response()
}

// ── DELETE /api/comment/:id ───────────────────────────────────────────────────

pub async fn delete_comment(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return Json(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    let is_admin = svc::is_admin(&state.db, user_id).await.unwrap_or(false);

    let existing = match comment::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(c)) => c,
        Ok(None) => return AppError::NotFound.into_response(),
        Err(e) => return AppError::from(e).into_response(),
    };

    if !is_admin && existing.user_id != Some(user_id) {
        return Json(json!({ "errno": 403, "errmsg": "Forbidden" })).into_response();
    }

    // Delete this comment and all replies
    let _ = comment::Entity::delete_many()
        .filter(
            comment::Column::Id.eq(id)
                .or(comment::Column::Pid.eq(id))
                .or(comment::Column::Rid.eq(id))
        )
        .exec(&state.db)
        .await;

    Json(json!({ "errno": 0, "errmsg": "" })).into_response()
}

// ── helpers ───────────────────────────────────────────────────────────────────

async fn fetch_comment(state: &AppState, id: i64) -> Result<Option<comment::Model>, AppError> {
    Ok(comment::Entity::find_by_id(id).one(&state.db).await?)
}

async fn build_comment_list(
    state: &AppState,
    comments: &[comment::Model],
    is_admin: bool,
    is_logged_in: bool,
) -> Result<Value, AppError> {
    let levels: Option<Vec<i64>> = state.config.levels.as_deref()
        .map(|s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect());
    let mut data = Vec::new();
    for c in comments {
        let user = if let Some(uid) = c.user_id {
            svc::get_user_by_id(&state.db, uid).await?
        } else {
            None
        };
        let level = if let Some(ref lvls) = levels {
            Some(compute_level(&state.db, c.user_id, c.mail.as_deref())
                .await
                .map(|cnt| svc::get_level(cnt, lvls))
                .unwrap_or(0))
        } else {
            None
        };
        data.push(svc::format_comment_with_opts(c, user.as_ref(), &state.config, is_admin, is_logged_in, level));
    }
    Ok(json!(data))
}

fn apply_status_filter(
    query: &mut sea_orm::Select<comment::Entity>,
    is_admin: bool,
    user_id: Option<i64>,
) {
    if is_admin {
        return;
    }
    match user_id {
        Some(id) => {
            *query = query.clone().filter(
                comment::Column::Status.is_not_in(["waiting", "spam"])
                    .or(comment::Column::UserId.eq(id))
            );
        }
        None => {
            *query = query.clone().filter(comment::Column::Status.is_not_in(["waiting", "spam"]));
        }
    }
}

async fn compute_level(
    db: &crate::db::Db,
    user_id: Option<i64>,
    mail: Option<&str>,
) -> Result<i64, AppError> {
    let count = if let Some(uid) = user_id {
        comment::Entity::find()
            .filter(comment::Column::UserId.eq(uid))
            .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
            .count(db)
            .await? as i64
    } else if let Some(m) = mail {
        comment::Entity::find()
            .filter(comment::Column::Mail.eq(m))
            .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
            .count(db)
            .await? as i64
    } else {
        0
    };
    Ok(count)
}
