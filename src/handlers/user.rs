use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::AppError,
    response::Json as JsonResponse,
    services::user as svc,
    state::AppState,
    utils::{extract_token, jwt},
};

fn auth(headers: &HeaderMap, secret: &str) -> Option<i64> {
    extract_token(headers).as_deref()
        .and_then(|t| jwt::object_id_from_token(t, secret).ok())
}

// ── POST /api/user ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub display_name: Option<String>,
    pub email: String,
    pub password: String,
    pub url: Option<String>,
}

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> impl IntoResponse {
    let host = headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or("localhost");
    let proto = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()).unwrap_or("http");
    let server_url = state.config.server_url.clone()
        .unwrap_or_else(|| format!("{proto}://{host}"));

    match svc::register(
        &state.db,
        body.display_name.as_deref().unwrap_or(&body.email),
        &body.email, &body.password, body.url.as_deref(),
        state.config.has_smtp(), &server_url,
    ).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(AppError::UserRegistered) => JsonResponse(json!({ "errno": 1000, "errmsg": "USER_EXIST" })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── POST /api/token ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub code: String,
}

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginBody>) -> impl IntoResponse {
    match svc::login(&state.db, &body.email, &body.password, &body.code,
        &state.config.jwt_secret(), state.config.avatar_proxy_url()).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(AppError::TwoFactorAuth) => JsonResponse(json!({ "errno": 1000, "errmsg": "TWO_FACTOR_AUTH_ERROR_DETAIL" })).into_response(),
        Err(_) => JsonResponse(json!({ "errno": 1000, "errmsg": "" })).into_response(),
    }
}

// ── DELETE /api/token ─────────────────────────────────────────────────────────

pub async fn logout() -> impl IntoResponse {
    JsonResponse(json!({ "errno": 0, "errmsg": "" }))
}

// ── GET /api/token ────────────────────────────────────────────────────────────

pub async fn get_token(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    match svc::get_profile(&state.db, user_id, state.config.avatar_proxy_url()).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── GET /api/user ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetUserQuery {
    pub email: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(rename = "pageSize", default = "default_page_size")]
    pub page_size: i64,
}
fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 10 }

pub async fn get_user(headers: HeaderMap, State(state): State<AppState>, Query(q): Query<GetUserQuery>) -> impl IntoResponse {
    let user_id = auth(&headers, &state.config.jwt_secret());
    let is_admin = if let Some(id) = user_id {
        crate::services::comment::is_admin(&state.db, id).await.unwrap_or(false)
    } else { false };

    if is_admin {
        // Admin: paginated list or single by email
        if let Some(email) = &q.email {
            match svc::get_by_email(&state.db, email).await {
                Ok(Some(u)) => return JsonResponse(json!({ "errno": 0, "errmsg": "", "data": svc::build_avatar(&u, state.config.avatar_proxy_url()) })).into_response(),
                Ok(None) => return JsonResponse(json!({ "errno": 0, "errmsg": "", "data": null })).into_response(),
                Err(e) => return e.into_response(),
            }
        }
        match svc::list_users(&state.db, q.page, q.page_size, state.config.avatar_proxy_url()).await {
            Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
            Err(e) => e.into_response(),
        }
    } else {
        // Public: top commenters by count
        let levels = state.config.levels.as_deref()
            .map(|s| s.split(',').filter_map(|v| v.trim().parse::<i64>().ok()).collect::<Vec<_>>());
        match svc::public_user_list(&state.db, q.page_size, levels.as_deref(), state.config.avatar_proxy_url()).await {
            Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
            Err(e) => e.into_response(),
        }
    }
}

// ── PUT /api/user ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateProfileBody {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub label: Option<String>,
    pub url: Option<String>,
    pub password: Option<String>,
    pub avatar: Option<String>,
    #[serde(rename = "2fa")]
    pub two_factor_auth: Option<String>,
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub google: Option<String>,
    pub weibo: Option<String>,
    pub qq: Option<String>,
    pub oidc: Option<String>,
    pub huawei: Option<String>,
    // admin sets type via this field when id is in path
    #[allow(dead_code)]
    #[serde(rename = "type")]
    pub user_type: Option<String>,
}

pub async fn update_profile(headers: HeaderMap, State(state): State<AppState>, Json(body): Json<UpdateProfileBody>) -> impl IntoResponse {
    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    let fields = svc::UpdateFields {
        display_name: body.display_name,
        email: body.email,
        label: body.label,
        url: body.url,
        password: body.password,
        avatar: body.avatar,
        two_factor_auth: body.two_factor_auth,
        github: body.github,
        twitter: body.twitter,
        facebook: body.facebook,
        google: body.google,
        weibo: body.weibo,
        qq: body.qq,
        oidc: body.oidc,
        huawei: body.huawei,
        user_type: None, // type change only via PUT /api/user/:id
    };
    match svc::update_profile(&state.db, user_id, None, fields).await {
        Ok(_) => JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── PUT /api/user/:id ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SetUserTypeBody {
    #[serde(rename = "type")]
    pub user_type: String,
}

pub async fn set_user_type(headers: HeaderMap, State(state): State<AppState>, Path(target_id): Path<i64>, Json(body): Json<SetUserTypeBody>) -> impl IntoResponse {
    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    if !crate::services::comment::is_admin(&state.db, user_id).await.unwrap_or(false) {
        return JsonResponse(json!({ "errno": 403, "errmsg": "Forbidden" })).into_response();
    }
    let fields = svc::UpdateFields { user_type: Some(body.user_type), ..Default::default() };
    match svc::update_profile(&state.db, user_id, Some(target_id), fields).await {
        Ok(_) => JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── DELETE /api/user/:id ──────────────────────────────────────────────────────

pub async fn delete_user(headers: HeaderMap, State(state): State<AppState>, Path(target_id): Path<i64>) -> impl IntoResponse {
    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    match svc::delete_user(&state.db, user_id, target_id).await {
        Ok(_) => JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── GET /api/token/2fa ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Get2faQuery { pub email: Option<String> }

pub async fn get_2fa(headers: HeaderMap, State(state): State<AppState>, Query(q): Query<Get2faQuery>) -> impl IntoResponse {
    let user_id = auth(&headers, &state.config.jwt_secret());
    match svc::get_2fa(&state.db, user_id, q.email.as_deref()).await {
        Ok(data) => JsonResponse(json!({ "errno": 0, "errmsg": "", "data": data })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── POST /api/token/2fa ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Set2faBody { pub code: String, pub secret: String }

pub async fn set_2fa(headers: HeaderMap, State(state): State<AppState>, Json(body): Json<Set2faBody>) -> impl IntoResponse {
    let user_id = match auth(&headers, &state.config.jwt_secret()) {
        Some(id) => id,
        None => return JsonResponse(json!({ "errno": 401, "errmsg": "Unauthorized" })).into_response(),
    };
    match svc::enable_2fa(&state.db, user_id, &body.secret, &body.code).await {
        Ok(_) => JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response(),
        Err(AppError::TwoFactorAuth) => JsonResponse(json!({ "errno": 1000, "errmsg": "TWO_FACTOR_AUTH_ERROR_DETAIL" })).into_response(),
        Err(e) => e.into_response(),
    }
}

// ── PUT /api/user/password ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PasswordResetBody { pub email: String }

pub async fn reset_password(headers: HeaderMap, State(state): State<AppState>, Json(body): Json<PasswordResetBody>) -> impl IntoResponse {
    if !state.config.has_smtp() {
        return JsonResponse(json!({ "errno": 1000, "errmsg": "" })).into_response();
    }
    let origin = headers.get("origin").and_then(|v| v.to_str().ok())
        .or(state.config.server_url.as_deref()).unwrap_or("").to_string();

    match svc::reset_password_request(&state.db, &body.email, &state.config.jwt_secret(), &origin).await {
        Ok(url) => {
            let config = state.config.clone();
            let email = body.email.clone();
            tokio::spawn(async move {
                crate::services::notify::email::send_mail(&config, &email,
                    "Reset Password",
                    "Please click <a href=\"{{url}}\">{{url}}</a> to login and change your password as soon as possible!",
                    serde_json::json!({ "url": url })).await;
            });
            JsonResponse(json!({ "errno": 0, "errmsg": "" })).into_response()
        }
        Err(e) => e.into_response(),
    }
}
