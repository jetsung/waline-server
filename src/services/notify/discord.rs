use super::NotifyContext;

/// Discord Webhook notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let webhook = match &ctx.config.discord_webhook {
        Some(w) if !w.is_empty() => w.clone(),
        _ => return false,
    };

    let template = ctx.config.discord_template.as_deref().unwrap_or(
        "💬 {{site.name}} 有新评论啦\n【评论者昵称】：{{self.nick}}\n【内容】：{{self.comment}}\n【地址】：{{site.postUrl}}"
    );
    let data = ctx.template_data();
    let content = render(template, &data);
    let title = format!("{} 有新评论", ctx.site_name());
    let full = format!("{title}\n{content}");

    let client = reqwest::Client::new();
    match client
        .post(&webhook)
        .form(&[("content", full.as_str())])
        .send()
        .await
    {
        Ok(_) => { tracing::info!("Discord notification sent"); true }
        Err(e) => { tracing::warn!("Discord failed: {e}"); false }
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
