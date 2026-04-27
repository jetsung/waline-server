use super::NotifyContext;
use serde_json::json;

/// 飞书/Lark Webhook notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let webhook = match &ctx.config.lark_webhook {
        Some(w) if !w.is_empty() => w.clone(),
        _ => return false,
    };

    let template = ctx.config.lark_template.as_deref().unwrap_or(
        "【网站名称】：{{site.name}}\n【评论者昵称】：{{self.nick}}\n【内容】：{{self.comment}}\n【地址】：{{site.postUrl}}"
    );
    let data = ctx.template_data();
    let content = render(template, &data);
    let title = format!("{} 有新评论", ctx.site_name());

    let mut body = json!({
        "msg_type": "post",
        "content": {
            "post": {
                "en_us": {
                    "title": title,
                    "content": [[{"tag": "text", "text": content}]]
                }
            }
        }
    });

    // Add signature if LARK_SECRET is configured
    if let Some(secret) = &ctx.config.lark_secret {
        if !secret.is_empty() {
            let timestamp = chrono::Utc::now().timestamp();
            let sign = lark_sign(timestamp, secret);
            body["timestamp"] = json!(timestamp);
            body["sign"] = json!(sign);
        }
    }

    let client = reqwest::Client::new();
    match client.post(&webhook).json(&body).send().await {
        Ok(_) => { tracing::info!("Lark notification sent"); true }
        Err(e) => { tracing::warn!("Lark failed: {e}"); false }
    }
}

fn lark_sign(timestamp: i64, secret: &str) -> String {
    use hmac::{Hmac, Mac, KeyInit};
    use sha2::Sha256;
    use base64::{Engine, engine::general_purpose::STANDARD};

    let sign_str = format!("{timestamp}\n{secret}");
    let mac = Hmac::<Sha256>::new_from_slice(sign_str.as_bytes()).unwrap();
    let result = mac.finalize().into_bytes();
    STANDARD.encode(result)
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
