use super::NotifyContext;

/// Server酱 (SC) push notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let key = match &ctx.config.sc_key {
        Some(k) if !k.is_empty() => k.clone(),
        _ => return false,
    };

    let template = ctx.config.sc_template.as_deref().unwrap_or(
        "{{site.name}} 有新评论啦\n【评论者昵称】：{{self.nick}}\n【内容】：{{self.comment}}\n【地址】：{{site.postUrl}}"
    );
    let data = ctx.template_data();
    let text = render(template, &data);
    let title = format!("{} 有新评论", ctx.site_name());

    let client = reqwest::Client::new();
    let url = format!("https://sctapi.ftqq.com/{key}.send");
    match client.post(&url).form(&[("text", &title), ("desp", &text)]).send().await {
        Ok(_) => { tracing::info!("Server酱 notification sent"); true }
        Err(e) => { tracing::warn!("Server酱 failed: {e}"); false }
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
