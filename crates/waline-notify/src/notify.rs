use waline_common::models::Comment;
use serde_json::Value;

/// Notification data passed to notification channels
#[derive(Debug, Clone)]
pub struct NotifyData {
    pub site_name: String,
    pub site_url: String,
    pub comment: Comment,
    pub parent_comment: Option<Comment>,
    pub is_admin_notify: bool,
}

/// Send notifications according to the original routing logic:
/// 1. If comment is not from author, notify via non-email channels. If no non-email channel, fall back to email.
/// 2. If comment is a reply, send email to parent commenter.
/// 3. DISABLE_AUTHOR_NOTIFY skips author notifications.
pub async fn send_notifications(
    data: &NotifyData,
    config: &waline_core::config::Config,
) -> Vec<Result<(), String>> {
    let mut results = Vec::new();

    // Author notification
    if !config.disable_author_notify && data.is_admin_notify {
        if config.has_non_email_channel() {
            // Send via non-email channels
            if let Some(ref key) = config.sc_key {
                results.push(send_sc(key, config.sc_template.as_deref(), data).await);
            }
            if let Some(ref key) = config.qywx_am {
                results.push(send_wecom(key, config.wx_template.as_deref(), data).await);
            }
            if let Some(ref key) = config.qmsg_key {
                results.push(send_qq(key, config.qq_id.as_deref(), config.qq_template.as_deref(), data).await);
            }
            if let (Some(ref token), Some(ref chat_id)) = (&config.tg_bot_token, &config.tg_chat_id) {
                results.push(send_telegram(token, chat_id, config.tg_template.as_deref(), data).await);
            }
            if let Some(ref key) = config.push_plus_key {
                results.push(send_pushplus(key, config.push_plus_topic.as_deref(), data).await);
            }
            if let Some(ref webhook) = config.discord_webhook {
                results.push(send_discord(webhook, config.discord_template.as_deref(), data).await);
            }
            if let Some(ref webhook) = config.lark_webhook {
                results.push(send_lark(webhook, config.lark_secret.as_deref(), config.lark_template.as_deref(), data).await);
            }
        } else if config.has_smtp() {
            // Fall back to email
            results.push(send_email(config, data).await);
        }
    }

    // Reply notification via email
    if let Some(ref _parent) = data.parent_comment {
        if config.has_smtp() {
            results.push(send_reply_email(config, data).await);
        }
    }

    results
}

/// Send email notification (author notification)
async fn send_email(config: &waline_core::config::Config, data: &NotifyData) -> Result<(), String> {
    let _ = (config, data);
    // TODO: implement SMTP email sending using lettre
    Ok(())
}

/// Send reply email notification
async fn send_reply_email(config: &waline_core::config::Config, data: &NotifyData) -> Result<(), String> {
    let _ = (config, data);
    // TODO: implement reply email
    Ok(())
}

/// Server酱 notification
async fn send_sc(key: &str, template: Option<&str>, data: &NotifyData) -> Result<(), String> {
    let client = reqwest::Client::new();
    let title = format!("{} 有新评论", data.site_name);
    let body = template
        .map(|t| render_template(t, data))
        .unwrap_or_else(|| format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment));

    client.post(format!("https://sctapi.ftqq.com/{key}.send"))
        .form(&[("title", &title), ("desp", &body)])
        .send().await
        .map_err(|e| format!("Server酱发送失败: {e}"))?;
    Ok(())
}

/// WeCom (企业微信) notification
async fn send_wecom(key: &str, template: Option<&str>, data: &NotifyData) -> Result<(), String> {
    let client = reqwest::Client::new();
    let content = template
        .map(|t| render_template(t, data))
        .unwrap_or_else(|| format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment));

    // Parse QYWX_AM: corpid,corpsecret,agentid
    let parts: Vec<&str> = key.split(',').collect();
    if parts.len() < 3 {
        return Err("QYWX_AM 格式错误，应为: corpid,corpsecret,agentid".to_string());
    }

    // Get access token
    let token_url = format!("https://qyapi.weixin.qq.com/cgi-bin/gettoken?corpid={}&corpsecret={}", parts[0], parts[1]);
    let resp: Value = client.get(&token_url).send().await.map_err(|e| format!("企业微信请求失败: {e}"))?
        .json().await.map_err(|e| format!("企业微信响应解析失败: {e}"))?;
    let access_token = resp["access_token"].as_str().ok_or("获取企业微信token失败")?;

    // Send message
    let msg_url = format!("https://qyapi.weixin.qq.com/cgi-bin/message/send?access_token={access_token}");
    let msg = serde_json::json!({
        "touser": "@all",
        "msgtype": "text",
        "agentid": parts[2],
        "text": { "content": content }
    });
    client.post(&msg_url).json(&msg).send().await
        .map_err(|e| format!("企业微信发送失败: {e}"))?;
    Ok(())
}

/// QQ (Qmsg) notification
async fn send_qq(key: &str, qq_id: Option<&str>, template: Option<&str>, data: &NotifyData) -> Result<(), String> {
    let client = reqwest::Client::new();
    let msg = template
        .map(|t| render_template(t, data))
        .unwrap_or_else(|| format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment));

    let mut form = vec![("msg", msg)];
    if let Some(qq) = qq_id {
        form.push(("qq", qq.to_string()));
    }

    client.post(format!("https://qmsg.zendee.cn/send/{key}"))
        .form(&form)
        .send().await
        .map_err(|e| format!("Qmsg发送失败: {e}"))?;
    Ok(())
}

/// Telegram notification
async fn send_telegram(token: &str, chat_id: &str, template: Option<&str>, data: &NotifyData) -> Result<(), String> {
    let client = reqwest::Client::new();
    let text = template
        .map(|t| render_template(t, data))
        .unwrap_or_else(|| format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment));

    client.post(format!("https://api.telegram.org/bot{token}/sendMessage"))
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "MarkdownV2"
        }))
        .send().await
        .map_err(|e| format!("Telegram发送失败: {e}"))?;
    Ok(())
}

/// PushPlus notification
async fn send_pushplus(key: &str, topic: Option<&str>, data: &NotifyData) -> Result<(), String> {
    let client = reqwest::Client::new();
    let title = format!("{} 有新评论", data.site_name);
    let content = format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment);

    let mut body = serde_json::json!({
        "token": key,
        "title": title,
        "content": content,
    });
    if let Some(t) = topic {
        body["topic"] = serde_json::Value::String(t.to_string());
    }

    client.post("https://www.pushplus.plus/send")
        .json(&body)
        .send().await
        .map_err(|e| format!("PushPlus发送失败: {e}"))?;
    Ok(())
}

/// Discord notification
async fn send_discord(webhook: &str, template: Option<&str>, data: &NotifyData) -> Result<(), String> {
    let client = reqwest::Client::new();
    let content = template
        .map(|t| render_template(t, data))
        .unwrap_or_else(|| format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment));

    client.post(webhook)
        .json(&serde_json::json!({ "content": content }))
        .send().await
        .map_err(|e| format!("Discord发送失败: {e}"))?;
    Ok(())
}

/// Lark/Feishu notification with HMAC signing
async fn send_lark(webhook: &str, secret: Option<&str>, template: Option<&str>, data: &NotifyData) -> Result<(), String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let client = reqwest::Client::new();
    let content = template
        .map(|t| render_template(t, data))
        .unwrap_or_else(|| format!("{} 评论了：{}", data.comment.nick.as_deref().unwrap_or("匿名"), data.comment.comment));

    let mut body = serde_json::json!({
        "msg_type": "text",
        "content": { "text": content }
    });

    if let Some(sec) = secret {
        let timestamp = chrono::Utc::now().timestamp();
        let string_to_sign = format!("{timestamp}\n{sec}");
        let mut mac = Hmac::<Sha256>::new_from_slice(sec.as_bytes())
            .map_err(|e| format!("HMAC初始化失败: {e}"))?;
        mac.update(string_to_sign.as_bytes());
        let sign = mac.finalize().into_bytes();
        let sign_str = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, sign);

        body["timestamp"] = serde_json::Value::String(timestamp.to_string());
        body["sign"] = serde_json::Value::String(sign_str);
    }

    client.post(webhook)
        .json(&body)
        .send().await
        .map_err(|e| format!("飞书发送失败: {e}"))?;
    Ok(())
}

/// Simple template rendering with variable substitution
fn render_template(template: &str, data: &NotifyData) -> String {
    let mut result = template.to_string();
    result = result.replace("{{site.name}}", &data.site_name);
    result = result.replace("{{site.url}}", &data.site_url);
    result = result.replace("{{comment.nick}}", data.comment.nick.as_deref().unwrap_or("匿名"));
    result = result.replace("{{comment.content}}", &data.comment.comment);
    result = result.replace("{{comment.url}}", data.comment.url.as_deref().unwrap_or(""));
    if let Some(ref mail) = data.comment.mail {
        result = result.replace("{{comment.mail}}", mail);
    }
    if let Some(ref ip) = data.comment.ip {
        result = result.replace("{{comment.ip}}", ip);
    }
    result
}
