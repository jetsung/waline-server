use waline_common::models::Comment;
use waline_core::config::Config;

/// Send webhook notification for a new comment
pub async fn send_webhook(comment: &Comment, config: &Config) -> Result<(), String> {
    let webhook_url = config.webhook.as_deref().ok_or("WEBHOOK not set")?;

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "objectId": comment.object_id,
        "nick": comment.nick,
        "mail": comment.mail,
        "comment": comment.comment,
        "url": comment.url,
        "status": comment.status,
        "insertedAt": comment.inserted_at,
    });

    client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Webhook request failed: {e}"))?;

    Ok(())
}
