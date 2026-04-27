use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;

use crate::{models::user, services::user as svc, state::AppState, utils::jwt};

#[derive(Debug, Deserialize)]
pub struct OAuthQuery {
    #[serde(rename = "type")]
    pub oauth_type: String,
    pub redirect: Option<String>,
    pub state: Option<String>, // @waline/admin passes JWT token here
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    #[serde(rename = "type")]
    pub oauth_type: String,
    pub redirect: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OAuthUser {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub url: Option<String>,
}

/// GET /api/oauth - Redirect to OAuth provider
pub async fn oauth_redirect(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(q): Query<OAuthQuery>,
) -> Response {
    let server_url = get_server_url(&headers, &state);
    
    // Build callback URL with redirect and type params (same as Node.js version)
    let redirect_back = format!(
        "{server_url}/api/oauth/callback?redirect={}&type={}", 
        urlencoding::encode(q.redirect.as_deref().unwrap_or("")),
        q.oauth_type
    );

    // Support both Authorization header and query state param (@waline/admin passes JWT here)
    // If query.state is provided, use it directly (it's already a valid JWT)
    // Otherwise try to extract from Authorization header
    let state_param = if let Some(ref s) = q.state {
        if !s.is_empty() {
            // Validate the token first
            if jwt::object_id_from_token(s, &state.config.jwt_secret()).is_ok() {
                s.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        // Try Authorization header
        extract_token_from_state(&headers, &state)
            .and_then(|uid| jwt::sign(uid, &state.config.jwt_secret(), 3600).ok())
            .unwrap_or_default()
    };

    // Build OAuth URL
    let oauth_url = format!(
        "{}/{}?redirect={}&state={}", 
        state.config.oauth_url, 
        q.oauth_type,
        urlencoding::encode(&redirect_back),
        urlencoding::encode(&state_param)
    );

    tracing::info!("OAuth redirect to: {}", oauth_url);
    Redirect::temporary(&oauth_url).into_response()
}

/// GET /api/oauth/callback - Handle OAuth callback
pub async fn oauth_callback(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(q): Query<OAuthCallbackQuery>,
) -> Response {
    let code = match &q.code {
        Some(c) if !c.is_empty() => c,
        _ => return Json(json!({ "errno": 1, "errmsg": "Missing code" })).into_response(),
    };
    
    handle_oauth_callback(&headers, &state, code, &q).await
}

async fn handle_oauth_callback(
    _headers: &HeaderMap,
    state: &AppState,
    code: &str,
    q: &OAuthCallbackQuery,
) -> Response {
    let oauth_type = &q.oauth_type;
    let redirect = q.redirect.as_deref().unwrap_or("");
    let state_param = q.state.as_deref().unwrap_or("");

    tracing::info!("OAuth callback: type={}, code={}, redirect={}, state={}", oauth_type, code, redirect, state_param);

    // Fetch user info from OAuth service
    let fetch_url = format!("{}/{oauth_type}?code={code}&state={}", 
        state.config.oauth_url, 
        urlencoding::encode(state_param)
    );
    
    tracing::info!("Fetching OAuth user from: {}", fetch_url);
    
    let oauth_user: OAuthUser = match reqwest::Client::new()
        .get(&fetch_url)
        .header("user-agent", "@waline")
        .send()
        .await
    {
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            tracing::info!("OAuth response status: {}, body: {}", status, text);
            match serde_json::from_str(&text) {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Failed to parse OAuth user: {}", e);
                    return Json(json!({ "errno": 1, "errmsg": "OAuth fetch failed" })).into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!("OAuth request failed: {}", e);
            return Json(json!({ "errno": 1, "errmsg": "OAuth request failed" })).into_response();
        }
    };

    tracing::info!("OAuth user: id={}, name={:?}", oauth_user.id, oauth_user.name);

    if oauth_user.id.is_empty() {
        return Json(json!({ "errno": 1, "errmsg": "Invalid OAuth user" })).into_response();
    }

    // 1. Check if social account already linked to a user
    let oauth_column = match oauth_type.as_str() {
        "github" => user::Column::Github,
        "twitter" => user::Column::Twitter,
        "facebook" => user::Column::Facebook,
        "google" => user::Column::Google,
        "weibo" => user::Column::Weibo,
        "qq" => user::Column::Qq,
        "oidc" => user::Column::Oidc,
        "huawei" => user::Column::Huawei,
        _ => {
            tracing::error!("Unknown OAuth type: {}", oauth_type);
            return Json(json!({ "errno": 1, "errmsg": "Unknown OAuth type" })).into_response();
        }
    };

    let user_by_social = user::Entity::find()
        .filter(oauth_column.eq(&oauth_user.id))
        .one(&state.db)
        .await
        .ok()
        .flatten();

    if let Some(user) = user_by_social {
        tracing::info!("Found existing user linked to OAuth: {}", user.id);
        let token = jwt::sign(user.id, &state.config.jwt_secret(), 2592000)
            .unwrap_or_default();
        if !redirect.is_empty() {
            let sep = if redirect.contains('?') { "&" } else { "?" };
            return Redirect::temporary(&format!("{redirect}{sep}token={token}")).into_response();
        }
        // Redirect to /ui/login with token, frontend will handle the redirect to profile
        return Redirect::temporary(&format!("/ui/login?token={token}")).into_response();
    }

    // 2. Current logged-in user linking social account
    // state_param may be a raw JWT, or a JSON object containing the JWT (OIDC providers)
    let current_user_id = if !state_param.is_empty() {
        tracing::debug!("Parsing state_param: {}", state_param);
        
        // Try to parse as JWT directly
        if let Ok(uid) = jwt::object_id_from_token(state_param, &state.config.jwt_secret()) {
            tracing::debug!("Parsed as direct JWT, uid={}", uid);
            Some(uid)
        } else {
            // Try to parse as JSON (OIDC providers wrap state in JSON)
            #[derive(serde::Deserialize)]
            struct StateWrapper {
                state: Option<String>,
            }
            if let Ok(wrapper) = serde_json::from_str::<StateWrapper>(state_param) {
                tracing::debug!("Parsed as JSON, inner state={:?}", wrapper.state);
                if let Some(ref inner_state) = wrapper.state {
                    jwt::object_id_from_token(inner_state, &state.config.jwt_secret()).ok()
                } else {
                    None
                }
            } else {
                // Try base64 decode then parse
                tracing::debug!("Trying base64 decode");
                if let Ok(json_str) = base64_url_decode(state_param) {
                    tracing::debug!("Base64 decoded: {}", json_str);
                    if let Ok(wrapper) = serde_json::from_str::<StateWrapper>(&json_str) {
                        tracing::debug!("Parsed decoded JSON, inner state={:?}", wrapper.state);
                        if let Some(ref inner_state) = wrapper.state {
                            jwt::object_id_from_token(inner_state, &state.config.jwt_secret()).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    };
    
    if let Some(uid) = current_user_id {
        tracing::info!("Linking OAuth to current user: {}", uid);
        let current_user = user::Entity::find_by_id(uid)
            .one(&state.db)
            .await
            .ok()
            .flatten();

        let update_avatar = current_user
            .as_ref()
            .map(|u| u.avatar.is_none() || u.avatar.as_deref() == Some(""))
            .unwrap_or(false)
            && oauth_user.avatar.is_some();

        if let Some(existing) = current_user {
            let mut active: user::ActiveModel = existing.into();
            match oauth_type.as_str() {
                "github" => active.github = Set(Some(oauth_user.id.clone())),
                "twitter" => active.twitter = Set(Some(oauth_user.id.clone())),
                "facebook" => active.facebook = Set(Some(oauth_user.id.clone())),
                "google" => active.google = Set(Some(oauth_user.id.clone())),
                "weibo" => active.weibo = Set(Some(oauth_user.id.clone())),
                "qq" => active.qq = Set(Some(oauth_user.id.clone())),
                "oidc" => active.oidc = Set(Some(oauth_user.id.clone())),
                "huawei" => active.huawei = Set(Some(oauth_user.id.clone())),
                _ => {}
            }
            if update_avatar {
                active.avatar = Set(oauth_user.avatar.clone());
            }
            active.updated_at = Set(Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
            let _ = active.update(&state.db).await;
        }
        return Redirect::temporary("/ui/profile").into_response();
    }

    // 3. Create new account from OAuth
    tracing::info!("Creating new user from OAuth");
    let email = oauth_user.email.clone()
        .unwrap_or_else(|| format!("{}@mail.{oauth_type}", oauth_user.id));

    let count = svc::count(&state.db).await.unwrap_or(1);
    let user_type = if count == 0 { "administrator" } else { "guest" };
    let hashed = crate::utils::password::hash(&uuid::Uuid::new_v4().to_string()).unwrap_or_default();

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut active = user::ActiveModel {
        display_name: Set(oauth_user.name.clone().unwrap_or_else(|| email.clone())),
        email: Set(email),
        url: Set(oauth_user.url.clone()),
        avatar: Set(oauth_user.avatar.clone()),
        password: Set(hashed),
        user_type: Set(user_type.to_string()),
        created_at: Set(Some(now.clone())),
        updated_at: Set(Some(now)),
        ..Default::default()
    };

    match oauth_type.as_str() {
        "github" => active.github = Set(Some(oauth_user.id.clone())),
        "twitter" => active.twitter = Set(Some(oauth_user.id.clone())),
        "facebook" => active.facebook = Set(Some(oauth_user.id.clone())),
        "google" => active.google = Set(Some(oauth_user.id.clone())),
        "weibo" => active.weibo = Set(Some(oauth_user.id.clone())),
        "qq" => active.qq = Set(Some(oauth_user.id.clone())),
        "oidc" => active.oidc = Set(Some(oauth_user.id.clone())),
        "huawei" => active.huawei = Set(Some(oauth_user.id.clone())),
        _ => {}
    }

    let new_user = match active.insert(&state.db).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            return Redirect::temporary("/ui/login").into_response();
        }
    };

    tracing::info!("Created new user with id: {}", new_user.id);

    let token = jwt::sign(new_user.id, &state.config.jwt_secret(), 2592000).unwrap_or_default();
    
    if redirect.is_empty() {
        // Redirect to /ui/login with token, frontend will handle the redirect to profile
        return Redirect::temporary(&format!("/ui/login?token={token}")).into_response();
    }

    let sep = if redirect.contains('?') { "&" } else { "?" };
    tracing::info!("Redirecting to: {}{}token={}", redirect, sep, token);
    Redirect::temporary(&format!("{redirect}{sep}token={token}")).into_response()
}

fn get_server_url(headers: &HeaderMap, state: &AppState) -> String {
    if let Some(url) = &state.config.server_url { return url.clone(); }
    let proto = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok()).unwrap_or("http");
    let host = headers.get(axum::http::header::HOST).and_then(|v| v.to_str().ok()).unwrap_or("localhost");
    format!("{proto}://{host}")
}

fn extract_token_from_state(headers: &HeaderMap, state: &AppState) -> Option<i64> {
    crate::utils::extract_token(headers)
        .as_deref()
        .and_then(|t| jwt::object_id_from_token(t, &state.config.jwt_secret()).ok())
}

fn base64_url_decode(input: &str) -> Result<String, ()> {
    // Add padding if needed
    let mut s = input.to_string();
    let padding = 4 - (s.len() % 4);
    if padding != 4 {
        for _ in 0..padding {
            s.push('=');
        }
    }
    // Replace URL-safe chars
    s = s.replace('-', "+").replace('_', "/");
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&s)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .map_err(|_| ())
}
