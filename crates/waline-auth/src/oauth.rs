use serde::Deserialize;

/// OAuth user info from external OAuth service
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
}

/// Fetch user info from the external OAuth service.
/// The OAuth service (default: https://oauth.lithub.cc) handles the actual OAuth flow.
/// This function calls the service with the provider type and code.
pub async fn fetch_oauth_user(oauth_url: &str, provider: &str, code: &str) -> Result<OAuthUserInfo, String> {
    let url = format!("{oauth_url}/{provider}?code={code}");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("OAuth request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("OAuth service returned status: {}", resp.status()));
    }

    resp.json()
        .await
        .map_err(|e| format!("OAuth response parse failed: {e}"))
}

/// Build the OAuth redirect URL for a given provider.
pub fn get_oauth_redirect_url(oauth_url: &str, provider: &str) -> String {
    format!("{oauth_url}/{provider}")
}
