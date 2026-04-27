use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use serde_json::{Value, json};
use totp_rs::{Secret, TOTP};

use crate::{
    db::Db,
    error::AppError,
    models::user,
    utils::{avatar, jwt, password},
};

pub async fn get_by_id(db: &Db, id: i64) -> Result<Option<user::Model>, AppError> {
    Ok(user::Entity::find_by_id(id).one(db).await?)
}

pub async fn get_by_email(db: &Db, email: &str) -> Result<Option<user::Model>, AppError> {
    Ok(user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await?)
}

pub async fn is_admin_by_id(db: &Db, id: i64) -> Result<bool, AppError> {
    Ok(get_by_id(db, id).await?.map(|u| u.is_admin()).unwrap_or(false))
}

pub async fn count(db: &Db) -> Result<i64, AppError> {
    Ok(user::Entity::find().count(db).await? as i64)
}

/// POST /api/user — register
pub async fn register(
    db: &Db,
    display_name: &str,
    email: &str,
    raw_password: &str,
    url: Option<&str>,
    has_smtp: bool,
    _server_url: &str,
) -> Result<Value, AppError> {
    use crate::models::user::ActiveModel;

    let hashed = password::hash(raw_password)?;
    let existing = get_by_email(db, email).await?;
    let is_first = count(db).await? == 0;

    let user_type = if is_first {
        "administrator".to_string()
    } else if has_smtp {
        let token = gen_verify_token();
        format!("verify:{token}:{}", Utc::now().timestamp_millis() + 3_600_000)
    } else {
        "guest".to_string()
    };

    if let Some(user) = existing {
        if user.is_admin() || user.user_type == "guest" {
            return Err(AppError::UserRegistered);
        }
        // re-register: update
        let mut active: ActiveModel = user.into();
        active.display_name = Set(display_name.to_string());
        active.password = Set(hashed);
        active.url = Set(url.map(|s| s.to_string()));
        active.user_type = Set(user_type.clone());
        active.updated_at = Set(Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
        active.update(db).await?;
    } else {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let active = ActiveModel {
            display_name: Set(display_name.to_string()),
            email: Set(email.to_string()),
            password: Set(hashed),
            user_type: Set(user_type.clone()),
            url: Set(url.map(|s| s.to_string())),
            created_at: Set(Some(now.clone())),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        active.insert(db).await?;
    }

    let needs_verify = user_type.starts_with("verify:");
    Ok(json!({ "verify": needs_verify }))
}

/// POST /api/token — login; returns full user object + token (objectId-based JWT)
pub async fn login(
    db: &Db,
    email: &str,
    raw_password: &str,
    code: &str,
    jwt_secret: &str,
    avatar_proxy: Option<&str>,
) -> Result<Value, AppError> {
    let user = get_by_email(db, email).await?.ok_or(AppError::Unauthorized)?;

    if !user.is_verified() {
        return Err(AppError::Unauthorized);
    }
    if !password::verify(raw_password, &user.password)? {
        return Err(AppError::Unauthorized);
    }

    // 2FA
    if let Some(secret) = &user.two_factor_auth {
        if !secret.is_empty() {
            let raw =
                Secret::Encoded(secret.clone()).to_raw().map_err(|_| AppError::TwoFactorAuth)?;
            let mut totp = TOTP::default();
            totp.secret = raw.to_bytes().map_err(|_| AppError::TwoFactorAuth)?;
            if !totp
                .check_current(code)
                .map_err(|_| AppError::TwoFactorAuth)?
            {
                return Err(AppError::TwoFactorAuth);
            }
        }
    }

    let avatar_url = build_avatar(&user, avatar_proxy);
    let token = jwt::sign(user.id, jwt_secret, 2592000)?;

    Ok(json!({
        "objectId": user.id,
        "display_name": user.display_name,
        "email": user.email,
        "password": null,
        "type": user.user_type,
        "label": user.label,
        "url": user.url,
        "avatar": avatar_url,
        "github": user.github,
        "twitter": user.twitter,
        "facebook": user.facebook,
        "google": user.google,
        "weibo": user.weibo,
        "qq": user.qq,
        "oidc": user.oidc,
        "huawei": user.huawei,
        "2fa": user.two_factor_auth,
        "createdAt": user.created_at,
        "updatedAt": user.updated_at,
        "token": token,
    }))
}

/// GET /api/token — return current user info (from JWT objectId)
pub async fn get_profile(db: &Db, user_id: i64, avatar_proxy: Option<&str>) -> Result<Value, AppError> {
    let user = get_by_id(db, user_id).await?.ok_or(AppError::UserNotFound)?;
    let avatar_url = build_avatar(&user, avatar_proxy);
    Ok(json!({
        "objectId": user.id,
        "display_name": user.display_name,
        "email": user.email,
        "type": user.user_type,
        "label": user.label,
        "url": user.url,
        "avatar": avatar_url,
        "github": user.github,
        "twitter": user.twitter,
        "facebook": user.facebook,
        "google": user.google,
        "weibo": user.weibo,
        "qq": user.qq,
        "oidc": user.oidc,
        "huawei": user.huawei,
        "2fa": user.two_factor_auth,
        "createdAt": user.created_at,
        "updatedAt": user.updated_at,
    }))
}

/// PUT /api/user — update profile (self or admin sets type by id)
pub async fn update_profile(
    db: &Db,
    object_id: i64,
    target_id: Option<i64>,
    fields: UpdateFields,
) -> Result<(), AppError> {
    use crate::models::user::ActiveModel;

    let id = target_id.unwrap_or(object_id);
    let user = get_by_id(db, id).await?.ok_or(AppError::UserNotFound)?;

    // Check email uniqueness if changing email
    if let Some(ref new_email) = fields.email {
        let existing = get_by_email(db, new_email).await?;
        if existing.map(|u| u.id != id).unwrap_or(false) {
            return Err(AppError::Internal("Email already in use".into()));
        }
    }

    let new_password = if let Some(p) = &fields.password {
        password::hash(p)?
    } else {
        user.password.clone()
    };

    let mut active: ActiveModel = user.into();
    if let Some(v) = fields.display_name {
        active.display_name = Set(v);
    }
    if let Some(v) = fields.email {
        active.email = Set(v);
    }
    if let Some(v) = fields.url {
        active.url = Set(Some(v));
    }
    if let Some(v) = fields.avatar {
        active.avatar = Set(Some(v));
    }
    active.password = Set(new_password);
    if let Some(v) = fields.label {
        active.label = Set(Some(v));
    }
    if let Some(v) = fields.two_factor_auth {
        active.two_factor_auth = Set(Some(v));
    }
    if let Some(v) = fields.github {
        active.github = Set(Some(v));
    }
    if let Some(v) = fields.twitter {
        active.twitter = Set(Some(v));
    }
    if let Some(v) = fields.facebook {
        active.facebook = Set(Some(v));
    }
    if let Some(v) = fields.google {
        active.google = Set(Some(v));
    }
    if let Some(v) = fields.weibo {
        active.weibo = Set(Some(v));
    }
    if let Some(v) = fields.qq {
        active.qq = Set(Some(v));
    }
    if let Some(v) = fields.oidc {
        active.oidc = Set(Some(v));
    }
    if let Some(v) = fields.huawei {
        active.huawei = Set(Some(v));
    }
    if let Some(v) = fields.user_type {
        active.user_type = Set(v);
    }
    active.updated_at = Set(Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
    active.update(db).await?;

    Ok(())
}

/// DELETE /api/user/:id — ban verified users, delete unverified
pub async fn delete_user(db: &Db, admin_id: i64, target_id: i64) -> Result<(), AppError> {
    use crate::models::user::ActiveModel;

    if !is_admin_by_id(db, admin_id).await? {
        return Err(AppError::Forbidden);
    }
    let user = get_by_id(db, target_id).await?.ok_or(AppError::UserNotFound)?;
    if user.user_type.starts_with("verify:") {
        user.delete(db).await?;
    } else {
        let mut active: ActiveModel = user.into();
        active.user_type = Set("banned".to_string());
        active.updated_at = Set(Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
        active.update(db).await?;
    }
    Ok(())
}

/// GET /api/user (admin) — paginated user list
pub async fn list_users(
    db: &Db,
    page: i64,
    page_size: i64,
    avatar_proxy: Option<&str>,
) -> Result<Value, AppError> {
    let total = user::Entity::find().count(db).await? as i64;
    let users = user::Entity::find()
        .order_by_desc(user::Column::CreatedAt)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(db)
        .await?;

    let data: Vec<Value> = users
        .iter()
        .map(|u| {
            let av = build_avatar(u, avatar_proxy);
            json!({
                "objectId": u.id,
                "display_name": u.display_name,
                "email": u.email,
                "type": u.user_type,
                "label": u.label,
                "url": u.url,
                "avatar": av,
                "github": u.github,
                "twitter": u.twitter,
                "facebook": u.facebook,
                "google": u.google,
                "weibo": u.weibo,
                "qq": u.qq,
                "oidc": u.oidc,
                "huawei": u.huawei,
                "2fa": u.two_factor_auth,
                "createdAt": u.created_at,
                "updatedAt": u.updated_at,
            })
        })
        .collect();

    let total_pages = (total + page_size - 1) / page_size;
    Ok(json!({ "page": page, "totalPages": total_pages, "pageSize": page_size, "data": data }))
}

/// GET /api/verification
pub async fn verify_email(db: &Db, email: &str, token: &str) -> Result<(), AppError> {
    use crate::models::user::ActiveModel;

    let user = get_by_email(db, email).await?.ok_or(AppError::UserNotFound)?;
    let re = regex::Regex::new(r"^verify:(\d{4}):(\d+)$").unwrap();
    let caps = re.captures(&user.user_type).ok_or(AppError::TokenExpired)?;
    let stored = caps.get(1).unwrap().as_str();
    let expires: i64 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
    if token != stored || Utc::now().timestamp_millis() > expires {
        return Err(AppError::TokenExpired);
    }
    let mut active: ActiveModel = user.into();
    active.user_type = Set("guest".to_string());
    active.updated_at = Set(Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
    active.update(db).await?;
    Ok(())
}

/// GET /api/token/2fa
pub async fn get_2fa(
    db: &Db,
    user_id: Option<i64>,
    email: Option<&str>,
) -> Result<Value, AppError> {
    // Unauthenticated: check if 2FA enabled for given email (login page)
    if user_id.is_none() {
        if let Some(email) = email {
            let user = get_by_email(db, email).await?;
            let enabled = user
                .and_then(|u| u.two_factor_auth)
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            return Ok(json!({ "enable": enabled }));
        }
        return Err(AppError::Unauthorized);
    }

    let user = get_by_id(db, user_id.unwrap())
        .await?
        .ok_or(AppError::UserNotFound)?;
    let name = format!("waline_{}", user.id);

    if let Some(secret) = &user.two_factor_auth {
        if secret.len() == 32 {
            return Ok(json!({
                "otpauth_url": format!("otpauth://totp/{name}?secret={secret}"),
                "secret": secret,
            }));
        }
    }

    let raw = Secret::generate_secret();
    let totp = TOTP {
        account_name: name,
        secret: raw.to_bytes().map_err(|_| AppError::TwoFactorAuth)?,
        ..Default::default()
    };
    Ok(json!({ "otpauth_url": totp.get_url(), "secret": totp.get_secret_base32() }))
}

/// POST /api/token/2fa
pub async fn enable_2fa(db: &Db, user_id: i64, secret: &str, code: &str) -> Result<(), AppError> {
    use crate::models::user::ActiveModel;

    let raw = Secret::Encoded(secret.to_string())
        .to_raw()
        .map_err(|_| AppError::TwoFactorAuth)?;
    let mut totp = TOTP::default();
    totp.secret = raw.to_bytes().map_err(|_| AppError::TwoFactorAuth)?;
    if !totp
        .check_current(code)
        .map_err(|_| AppError::TwoFactorAuth)?
    {
        return Err(AppError::TwoFactorAuth);
    }
    let user = get_by_id(db, user_id)
        .await?
        .ok_or(AppError::UserNotFound)?;
    let mut active: ActiveModel = user.into();
    active.two_factor_auth = Set(Some(secret.to_string()));
    active.updated_at = Set(Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));
    active.update(db).await?;
    Ok(())
}

// --- helpers ---

pub fn build_avatar(user: &user::Model, proxy: Option<&str>) -> String {
    let url = user
        .avatar
        .clone()
        .unwrap_or_else(|| avatar::gravatar_url(&user.email, None));
    avatar::proxied_avatar(&url, proxy)
}

fn gen_verify_token() -> String {
    use rand::RngExt;
    format!("{:04}", rand::rng().random_range(0..10000u32))
}

#[derive(Default)]
pub struct UpdateFields {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub avatar: Option<String>,
    pub password: Option<String>,
    pub label: Option<String>,
    pub two_factor_auth: Option<String>,
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub facebook: Option<String>,
    pub google: Option<String>,
    pub weibo: Option<String>,
    pub qq: Option<String>,
    pub oidc: Option<String>,
    pub huawei: Option<String>,
    pub user_type: Option<String>,
}

pub async fn reset_password_request(
    db: &Db,
    email: &str,
    jwt_secret: &str,
    server_url: &str,
) -> Result<String, AppError> {
    let user = get_by_email(db, email).await?.ok_or(AppError::UserNotFound)?;
    // Node版: jwt.sign(user[0].objectId, jwtKey) — sign with objectId
    let token = jwt::sign(user.id, jwt_secret, 300)?;
    Ok(format!("{server_url}/ui/profile?token={token}"))
}

/// GET /api/user (public) — top commenters ranked by comment count
pub async fn public_user_list(
    db: &Db,
    limit: i64,
    levels: Option<&[i64]>,
    avatar_proxy: Option<&str>,
) -> Result<Value, AppError> {
    use crate::models::comment;
    use sea_orm::QuerySelect;

    // Get comment counts grouped by user_id and mail - use raw SQL for ordering by alias
    #[derive(Debug, sea_orm::FromQueryResult)]
    struct CountRow {
        user_id: Option<i64>,
        mail: Option<String>,
        count: i64,
    }

    let counts: Vec<CountRow> = comment::Entity::find()
        .select_only()
        .column(comment::Column::UserId)
        .column(comment::Column::Mail)
        .column_as(comment::Column::Id.count(), "count")
        .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
        .group_by(comment::Column::UserId)
        .group_by(comment::Column::Mail)
        .limit(limit as u64)
        .into_model::<CountRow>()
        .all(db)
        .await?;

    // Sort in memory by count descending
    let mut counts = counts;
    counts.sort_by(|a, b| b.count.cmp(&a.count));

    // Sort in memory by count descending
    let mut counts = counts;
    counts.sort_by(|a, b| b.count.cmp(&a.count));

    let mut result = Vec::new();

    for row in &counts {
        let mut entry = serde_json::Map::new();
        entry.insert("count".into(), json!(row.count));

        if let Some(lvls) = levels {
            entry.insert("level".into(), json!(get_level(row.count, lvls)));
        }

        // Try to get user info from Users table
        if let Some(uid) = row.user_id {
            if let Ok(Some(user)) = get_by_id(db, uid).await {
                let av = build_avatar(&user, avatar_proxy);
                entry.insert("nick".into(), json!(user.display_name));
                entry.insert("link".into(), json!(user.url));
                entry.insert("avatar".into(), json!(av));
                entry.insert("label".into(), json!(user.label));
                result.push(Value::Object(entry));
                continue;
            }
        }

        // Fall back to comment data
        if let Some(mail) = &row.mail {
            #[derive(Debug, sea_orm::FromQueryResult)]
            struct CmtRow {
                nick: Option<String>,
                link: Option<String>,
            }
            if let Ok(Some(cmt)) = comment::Entity::find()
                .select_only()
                .column(comment::Column::Nick)
                .column(comment::Column::Link)
                .filter(comment::Column::Mail.eq(mail))
                .into_model::<CmtRow>()
                .one(db)
                .await
            {
                let av = avatar::proxied_avatar(&avatar::gravatar_url(mail, None), avatar_proxy);
                entry.insert("nick".into(), json!(cmt.nick));
                entry.insert("link".into(), json!(cmt.link));
                entry.insert("avatar".into(), json!(av));
                result.push(Value::Object(entry));
            }
        }
    }

    Ok(json!(result))
}

fn get_level(count: i64, levels: &[i64]) -> i32 {
    let mut level = 0i32;
    for (i, &threshold) in levels.iter().enumerate() {
        if count >= threshold {
            level = i as i32;
        }
    }
    level
}
