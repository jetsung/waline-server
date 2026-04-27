use super::NotifyContext;

/// PushPlus notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let key = match &ctx.config.push_plus_key {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return false,
    };

    let data = ctx.template_data();
    let title = format!("{} 有新评论", ctx.site_name());
    let content = format!(
        "【评论者昵称】：{}<br>【内容】：{}<br>【地址】：{}",
        ctx.nick(),
        ctx.comment_text(),
        ctx.post_url()
    );

    let mut params = vec![
        ("token".to_string(), key),
        ("title".to_string(), title),
        ("content".to_string(), content),
    ];
    if let Some(topic) = &ctx.config.push_plus_topic {
        params.push(("topic".to_string(), topic.clone()));
    }
    if let Some(channel) = &ctx.config.push_plus_channel {
        params.push(("channel".to_string(), channel.clone()));
    }
    if let Some(webhook) = &ctx.config.push_plus_webhook {
        params.push(("webhook".to_string(), webhook.clone()));
    }

    let _ = data; // suppress unused warning
    let client = reqwest::Client::new();
    let form: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    match client.post("http://www.pushplus.plus/send").form(&form).send().await {
        Ok(_) => { tracing::info!("PushPlus notification sent"); true }
        Err(e) => { tracing::warn!("PushPlus failed: {e}"); false }
    }
}
