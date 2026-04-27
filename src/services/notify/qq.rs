use super::NotifyContext;

/// Qmsg QQ notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let key = match &ctx.config.qmsg_key {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return false,
    };
    let qq_id = ctx.config.qq_id.as_deref().unwrap_or("");

    let template = ctx.config.qq_template.as_deref().unwrap_or(
        "💬 {{site.name}} 有新评论啦\n{{self.nick}} 评论道：\n{{self.comment}}\n仅供预览，请前往查看完整内容。"
    );
    let data = ctx.template_data();
    let msg = render(template, &data);

    let client = reqwest::Client::new();
    let url = format!("https://qmsg.zendee.cn/send/{key}");
    match client.post(&url).form(&[("qq", qq_id), ("msg", &msg)]).send().await {
        Ok(_) => { tracing::info!("QQ notification sent"); true }
        Err(e) => { tracing::warn!("QQ notify failed: {e}"); false }
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
