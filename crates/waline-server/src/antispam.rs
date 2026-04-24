use waline_common::error::WalineError;
use waline_common::models::NewComment;
use waline_core::config::Config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Utc, DateTime};

/// Check comment against all anti-spam rules
pub async fn check_spam(
    comment: &NewComment,
    ip: &str,
    config: &Config,
) -> Result<(), WalineError> {
    // Check forbidden words
    check_forbidden_words(comment, config)?;

    // Check IP rate limiting
    check_ip_rate_limit(ip, config).await?;

    // Check CAPTCHA (if required)
    // CAPTCHA verification happens in the handler before this is called

    // Check Akismet (async, best-effort)
    if let Some(ref akismet_key) = config.akismet_key {
        if akismet_key != "false" {
            let _ = check_akismet(comment, ip, akismet_key).await;
        }
    }

    Ok(())
}

/// Check if comment contains forbidden words
fn check_forbidden_words(comment: &NewComment, config: &Config) -> Result<(), WalineError> {
    if config.forbidden_words.is_empty() {
        return Ok(());
    }
    let content_lower = comment.comment.to_lowercase();
    for word in &config.forbidden_words {
        if content_lower.contains(&word.to_lowercase()) {
            return Err(WalineError::SpamDetected);
        }
    }
    Ok(())
}

/// In-memory IP rate limiter
static IP_TIMESTAMPS: once_cell::sync::Lazy<Arc<RwLock<HashMap<String, DateTime<Utc>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Check IP rate limit
async fn check_ip_rate_limit(ip: &str, config: &Config) -> Result<(), WalineError> {
    let timestamps = IP_TIMESTAMPS.read().await;
    if let Some(last_time) = timestamps.get(ip) {
        let elapsed = Utc::now().timestamp() - last_time.timestamp();
        if elapsed < config.ipqps as i64 {
            return Err(WalineError::RateLimited);
        }
    }
    drop(timestamps);

    // Update timestamp
    let mut timestamps = IP_TIMESTAMPS.write().await;
    timestamps.insert(ip.to_string(), Utc::now());

    // Clean old entries periodically
    if timestamps.len() > 10000 {
        let cutoff = Utc::now() - chrono::Duration::seconds(config.ipqps as i64 * 2);
        timestamps.retain(|_, v| *v > cutoff);
    }

    Ok(())
}

/// Check comment against Akismet API
async fn check_akismet(
    comment: &NewComment,
    ip: &str,
    api_key: &str,
) -> Result<bool, String> {
    let client = reqwest::Client::new();
    let blog = std::env::var("SITE_URL").unwrap_or_default();

    let params = [
        ("api_key", api_key),
        ("blog", &blog),
        ("user_ip", ip),
        ("comment_type", "comment"),
        ("comment_author", comment.nick.as_deref().unwrap_or("")),
        ("comment_author_email", comment.mail.as_deref().unwrap_or("")),
        ("comment_content", &comment.comment),
    ];

    let resp = client
        .post(format!("https://{api_key}.rest.akismet.com/1.1/comment-check"))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Akismet request failed: {e}"))?;

    let body = resp.text().await.map_err(|e| format!("Akismet response failed: {e}"))?;
    Ok(body == "true")
}

/// Check for duplicate content from same IP
pub fn check_duplicate(comment: &NewComment, ip: &str, _config: &Config) -> Result<(), WalineError> {
    // TODO: implement duplicate content detection with a short-lived cache
    Ok(())
}

/// Verify CAPTCHA (reCAPTCHA v3 or Turnstile)
pub async fn verify_captcha(
    config: &Config,
    recaptcha_token: Option<&str>,
    turnstile_token: Option<&str>,
) -> Result<(), WalineError> {
    if let (Some(ref secret), Some(token)) = (&config.recaptcha_v3_secret, recaptcha_token) {
        waline_auth::captcha::verify_recaptcha(secret, token)
            .await
            .map_err(|e| WalineError::BadRequest(e))?;
    }
    if let (Some(ref secret), Some(token)) = (&config.turnstile_secret, turnstile_token) {
        waline_auth::captcha::verify_turnstile(secret, token)
            .await
            .map_err(|e| WalineError::BadRequest(e))?;
    }
    Ok(())
}
