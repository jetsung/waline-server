use serde_json::{Value, json};

use crate::{
    config::Config,
    db::Db,
    error::AppError,
    models::{comment, user},
    utils::{avatar, ip, ua},
};

/// Build the public-facing comment JSON (matches Node formatCmt).
/// `login_user_id`: Some(id) if a user is logged in (to expose `orig`)
/// `level`: optional pre-computed user level
pub fn format_comment(
    comment: &comment::Model,
    matched_user: Option<&user::Model>,
    config: &Config,
    is_admin: bool,
) -> Value {
    format_comment_with_opts(comment, matched_user, config, is_admin, false, None)
}

pub fn format_comment_with_opts(
    comment: &comment::Model,
    matched_user: Option<&user::Model>,
    config: &Config,
    is_admin: bool,
    is_logged_in: bool,
    level: Option<i32>,
) -> Value {
    let (browser, os) = if config.disable_useragent {
        (String::new(), String::new())
    } else {
        ua::parse(comment.ua.as_deref().unwrap_or(""))
    };

    let avatar_url = match matched_user.and_then(|u| u.avatar.as_deref()) {
        Some(a) if !a.is_empty() => a.to_string(),
        _ => avatar::gravatar_url(comment.mail.as_deref().unwrap_or(""), config.gravatar_str.as_deref()),
    };
    let avatar_url = avatar::proxied_avatar(&avatar_url, config.avatar_proxy_url());

    let addr = if is_admin || !config.disable_region {
        ip::lookup(comment.ip.as_deref().unwrap_or(""))
    } else {
        String::new()
    };

    let time = comment.time_ms();

    let mut v = json!({
        "objectId": comment.id,
        "comment": comment.comment,
        "insertedAt": comment.inserted_at,
        "nick": comment.nick,
        "link": comment.link,
        "avatar": avatar_url,
        "browser": browser,
        "os": os,
        "addr": addr,
        "time": time,
        "like": comment.like_count.unwrap_or(0),
        "status": comment.status,
        "sticky": comment.sticky.map(|s| s != 0).unwrap_or(false),
        "url": comment.url,
        "pid": comment.pid,
        "rid": comment.rid,
        "user_id": comment.user_id,
    });

    // orig: raw markdown visible to logged-in users
    if is_logged_in || is_admin {
        v["orig"] = json!(comment.comment);
    }

    if let Some(u) = matched_user {
        v["type"] = json!(u.user_type);
        v["label"] = json!(u.label);
        v["nick"] = json!(u.display_name);
        v["link"] = json!(u.url);
    }

    if is_admin {
        v["mail"] = json!(comment.mail);
        v["ip"] = json!(comment.ip);
    }

    if let Some(lvl) = level {
        v["level"] = json!(lvl);
    }

    v
}

pub fn get_level(count: i64, levels: &[i64]) -> i32 {
    let mut level = 0i32;
    for (i, &threshold) in levels.iter().enumerate() {
        if count >= threshold {
            level = i as i32;
        }
    }
    level
}

pub async fn get_user_by_id(db: &Db, id: i64) -> Result<Option<user::Model>, AppError> {
    super::user::get_by_id(db, id).await
}

pub async fn is_admin(db: &Db, user_id: i64) -> Result<bool, AppError> {
    super::user::is_admin_by_id(db, user_id).await
}

pub async fn send_webhook(url: &str, comment: &comment::Model) {
    let client = reqwest::Client::new();
    let body = json!({
        "event": "new_comment",
        "comment": {
            "objectId": comment.id,
            "nick": comment.nick,
            "mail": comment.mail,
            "comment": comment.comment,
            "url": comment.url,
            "status": comment.status,
        }
    });
    if let Err(e) = client.post(url).json(&body).send().await {
        tracing::warn!("Webhook failed: {e}");
    }
}
