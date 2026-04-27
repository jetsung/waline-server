/// Check if comment content contains forbidden words.
pub fn has_forbidden_word(content: &str, words: &[String]) -> bool {
    if words.is_empty() {
        return false;
    }
    let content_lower = content.to_lowercase();
    words.iter().any(|w| content_lower.contains(w.to_lowercase().as_str()))
}

/// Check comment via Akismet API. Returns true if spam.
pub async fn akismet_check(
    key: &str,
    blog: &str,
    ip: &str,
    ua: &str,
    author: &str,
    email: &str,
    content: &str,
) -> bool {
    if key.is_empty() || key == "false" {
        return false;
    }
    let client = reqwest::Client::new();
    let params = [
        ("blog", blog),
        ("user_ip", ip),
        ("user_agent", ua),
        ("comment_author", author),
        ("comment_author_email", email),
        ("comment_content", content),
        ("comment_type", "comment"),
    ];
    let url = format!("https://{key}.rest.akismet.com/1.1/comment-check");
    match client.post(&url).form(&params).send().await {
        Ok(resp) => resp.text().await.map(|t| t.trim() == "true").unwrap_or(false),
        Err(e) => {
            tracing::warn!("Akismet check failed: {e}");
            false
        }
    }
}

/// Verify reCAPTCHA v3 token. Returns true if valid.
pub async fn verify_recaptcha(secret: &str, token: &str) -> bool {
    if secret.is_empty() || token.is_empty() {
        return true; // not configured, skip
    }
    let client = reqwest::Client::new();
    let params = [("secret", secret), ("response", token)];
    match client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&params)
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<serde_json::Value>()
            .await
            .map(|v| v["success"].as_bool().unwrap_or(false))
            .unwrap_or(false),
        Err(_) => false,
    }
}

/// Verify Cloudflare Turnstile token. Returns true if valid.
pub async fn verify_turnstile(secret: &str, token: &str) -> bool {
    if secret.is_empty() || token.is_empty() {
        return true;
    }
    let client = reqwest::Client::new();
    let params = [("secret", secret), ("response", token)];
    match client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&params)
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<serde_json::Value>()
            .await
            .map(|v| v["success"].as_bool().unwrap_or(false))
            .unwrap_or(false),
        Err(_) => false,
    }
}
