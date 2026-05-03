use axum::{
    extract::{Query, State},
    http::{HeaderMap, header},
    response::{Html, IntoResponse},
};
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UiQuery {
    #[allow(dead_code)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct OAuthService {
    name: String,
    origin: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OAuthServicesResponse {
    services: Vec<OAuthService>,
}

async fn get_oauth_services(oauth_url: &str) -> Vec<OAuthService> {
    match reqwest::Client::new()
        .get(oauth_url)
        .header("user-agent", "@waline")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            match r.json::<OAuthServicesResponse>().await {
                Ok(resp) => resp.services,
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

fn admin_html(
    site_url: &str, 
    site_name: &str, 
    server_url: &str, 
    config: &crate::config::Config,
    oauth_services: &[OAuthService],
) -> String {
    let recaptcha_v3_key = config.recaptcha_v3_key.as_deref()
        .map(|k| format!("'{k}'"))
        .unwrap_or_else(|| "undefined".to_string());
    let turnstile_key = config.turnstile_key.as_deref()
        .map(|k| format!("'{k}'"))
        .unwrap_or_else(|| "undefined".to_string());
    let asset_url = &config.waline_admin_module_asset_url;
    
    let oauth_services_js = serde_json::to_string(oauth_services).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Waline Management System</title>
    <meta name="viewport" content="width=device-width,initial-scale=1">
  </head>
  <body>
    <script>
    window.SITE_URL = `{site_url}`;
    window.SITE_NAME = `{site_name}`;
    window.recaptchaV3Key = {recaptcha_v3_key};
    window.turnstileKey = {turnstile_key};
    window.oauthServices = {oauth_services_js};
    window.serverURL = '{server_url}/api/';
    </script>
    <script src="{asset_url}"></script>
  </body>
</html>"#
    )
}

fn get_server_url(headers: &HeaderMap, config: &crate::config::Config) -> String {
    if let Some(url) = &config.server_url {
        return url.clone();
    }
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    format!("{proto}://{host}")
}

/// GET /ui - Admin UI entry point
/// Frontend (@waline/admin) handles token from URL parameter
pub async fn ui_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(_query): Query<UiQuery>,
) -> impl IntoResponse {
    let server_url = get_server_url(&headers, &state.config);
    let site_url = state.config.site_url.as_deref().unwrap_or("");
    let site_name = state.config.site_name.as_deref().unwrap_or("");
    let oauth_services = get_oauth_services(&state.config.oauth_url).await;
    
    Html(admin_html(site_url, site_name, &server_url, &state.config, &oauth_services)).into_response()
}
