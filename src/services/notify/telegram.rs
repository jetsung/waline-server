use super::NotifyContext;
use serde_json::json;

/// Telegram Bot notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let bot_token = match &ctx.config.tg_bot_token {
        Some(t) if !t.is_empty() => t.clone(),
        _ => return false,
    };
    let chat_id = match &ctx.config.tg_chat_id {
        Some(c) if !c.is_empty() => c.clone(),
        _ => return false,
    };

    let template = ctx.config.tg_template.as_deref().unwrap_or(
        "💬 *{{site.name}} 有新评论啦*\n\n*{{self.nick}}* 评论道：\n```\n{{self.comment}}\n```\n*邮箱：*`{{self.mail}}`\n\n点击[查看完整内容]({{site.postUrl}})"
    );
    let data = ctx.template_data();
    let text = render(template, &data);

    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let body = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "MarkdownV2",
    });

    match client.post(&url).json(&body).send().await {
        Ok(r) => {
            let ok = r.json::<serde_json::Value>().await
                .map(|v| v["ok"].as_bool().unwrap_or(false))
                .unwrap_or(false);
            if ok { tracing::info!("Telegram notification sent"); }
            ok
        }
        Err(e) => { tracing::warn!("Telegram failed: {e}"); false }
    }
}

fn render(template: &str, data: &serde_json::Value) -> String {
    let mut s = template.to_string();
    if let Some(site) = data.get("site").and_then(|v| v.as_object()) {
        for (k, v) in site {
            s = s.replace(&format!("{{{{site.{k}}}}}"), &v.as_str().unwrap_or("").to_string());
        }
    }
    if let Some(slf) = data.get("self").and_then(|v| v.as_object()) {
        for (k, v) in slf {
            s = s.replace(&format!("{{{{self.{k}}}}}"), &v.as_str().unwrap_or("").to_string());
        }
    }
    s
}
