use super::NotifyContext;
use serde_json::json;

/// 企业微信 (QYWX_AM) notification. Returns true if sent.
pub async fn send(ctx: &NotifyContext<'_>) -> bool {
    let qywx_am = match &ctx.config.qywx_am {
        Some(v) if !v.is_empty() => v.clone(),
        _ => return false,
    };

    let parts: Vec<&str> = qywx_am.split(',').collect();
    if parts.len() < 4 {
        return false;
    }
    let (corp_id, corp_secret, to_user, agent_id) = (parts[0], parts[1], parts[2], parts[3]);
    let thumb_media_id = parts.get(4).copied().unwrap_or("");

    let template = ctx.config.wx_template.as_deref().unwrap_or(
        "💬 {{site.name}} 的文章有新评论啦\n【评论者昵称】：{{self.nick}}\n【内容】：{{self.comment}}\n【地址】：{{site.postUrl}}"
    );
    let data = ctx.template_data();
    let content = render(template, &data);
    let title = format!("{} 有新评论", ctx.site_name());

    let client = reqwest::Client::new();
    let token_url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={corp_id}&corpsecret={corp_secret}"
    );
    let token_resp: serde_json::Value = match client.get(&token_url).send().await {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => return false,
    };
    let access_token = match token_resp["access_token"].as_str() {
        Some(t) => t.to_string(),
        None => return false,
    };

    let send_url = format!(
        "https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={access_token}"
    );
    let body = json!({
        "touser": to_user,
        "agentid": agent_id,
        "msgtype": "mpnews",
        "mpnews": {
            "articles": [{
                "title": title,
                "thumb_media_id": thumb_media_id,
                "author": "Waline Comment",
                "content_source_url": ctx.post_url(),
                "content": content,
                "digest": content,
            }]
        }
    });

    match client.post(&send_url).json(&body).send().await {
        Ok(_) => { tracing::info!("企业微信 notification sent"); true }
        Err(e) => { tracing::warn!("企业微信 failed: {e}"); false }
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
