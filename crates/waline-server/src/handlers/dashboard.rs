use axum::extract::State;
use axum::response::Html;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthService {
    name: String,
    origin: String,
}

#[derive(Debug, Deserialize)]
struct OAuthResponse {
    services: Vec<OAuthService>,
}

/// GET /ui/* - Dashboard (admin UI) HTML shell
///
/// Serves the same HTML for all /ui/* paths. The @waline/admin React SPA
/// handles client-side routing for /ui/login, /ui/profile, etc.
pub async fn dashboard(State(state): State<Arc<AppState>>) -> Html<String> {
    let config = &state.config;

    // Fetch OAuth services from external provider
    let oauth_services = fetch_oauth_services(&config.oauth_url).await;

    let site_url = serde_json::to_string(&config.site_url).unwrap_or_else(|_| "null".into());
    let site_name = serde_json::to_string(&config.site_name).unwrap_or_else(|_| "null".into());
    let recaptcha_v3_key =
        serde_json::to_string(&config.recaptcha_v3_key).unwrap_or_else(|_| "null".into());
    let turnstile_key =
        serde_json::to_string(&config.turnstile_key).unwrap_or_else(|_| "null".into());
    let oauth_json = serde_json::to_string(&oauth_services).unwrap_or_else(|_| "[]".into());

    // Determine serverURL: use SERVER_URL env or derive from request
    let server_url = config
        .server_url
        .as_deref()
        .unwrap_or("");

    let admin_script_url = config
        .waline_admin_module_asset_url
        .as_deref()
        .unwrap_or("//unpkg.com/@waline/admin");

    let html = format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Waline Management System</title>
    <meta name="viewport" content="width=device-width,initial-scale=1">
  </head>
  <body>
    <script>
    window.SITE_URL = {site_url};
    window.SITE_NAME = {site_name};
    window.recaptchaV3Key = {recaptcha_v3_key};
    window.turnstileKey = {turnstile_key};
    window.oauthServices = {oauth_json};
    window.serverURL = '{server_url}/api/';
    </script>
    <script src="{admin_script_url}"></script>
  </body>
</html>"#
    );

    Html(html)
}

async fn fetch_oauth_services(oauth_url: &str) -> Vec<OAuthService> {
    let client = reqwest::Client::new();
    match client
        .get(oauth_url)
        .header("user-agent", "@waline")
        .send()
        .await
    {
        Ok(resp) => match resp.json::<OAuthResponse>().await {
            Ok(r) => r.services,
            Err(_) => vec![],
        },
        Err(_) => vec![],
    }
}
