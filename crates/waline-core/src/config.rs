use serde::Deserialize;
use std::env;

/// Database type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgresql,
    Mysql,
    Sqlite,
}

/// Main application configuration, loaded from environment variables.
/// All variable names match the original Waline Node.js server.
#[derive(Debug, Clone)]
pub struct Config {
    // Database - PostgreSQL
    pub pg_db: Option<String>,
    pub pg_host: Option<String>,
    pub pg_port: Option<u16>,
    pub pg_user: Option<String>,
    pub pg_password: Option<String>,
    pub pg_prefix: Option<String>,
    pub pg_ssl: Option<bool>,
    pub postgres_url: Option<String>,

    // Database - MySQL
    pub mysql_db: Option<String>,
    pub mysql_host: Option<String>,
    pub mysql_port: Option<u16>,
    pub mysql_user: Option<String>,
    pub mysql_password: Option<String>,
    pub mysql_prefix: Option<String>,
    pub mysql_charset: Option<String>,
    pub mysql_ssl: Option<bool>,

    // Database - SQLite
    pub sqlite_path: Option<String>,
    pub sqlite_prefix: Option<String>,

    // JWT
    pub jwt_token: Option<String>,
    pub jwt_key: String,

    // Security
    pub secure_domains: Option<Vec<String>>,
    pub disable_useragent: bool,
    pub disable_region: bool,
    pub forbidden_words: Vec<String>,
    pub ipqps: u64,
    pub comment_audit: bool,
    pub login: Option<String>,
    pub akismet_key: Option<String>,
    pub recaptcha_v3_secret: Option<String>,
    pub recaptcha_v3_key: Option<String>,
    pub turnstile_secret: Option<String>,
    pub turnstile_key: Option<String>,

    // Site
    pub site_name: Option<String>,
    pub site_url: Option<String>,
    pub server_url: Option<String>,

    // SMTP / Email
    pub smtp_service: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_secure: Option<bool>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub sender_email: Option<String>,
    pub sender_name: Option<String>,
    pub author_email: Option<String>,
    pub disable_author_notify: bool,

    // Email templates
    pub mail_subject: Option<String>,
    pub mail_template: Option<String>,
    pub mail_subject_admin: Option<String>,
    pub mail_template_admin: Option<String>,

    // Notification channels
    pub sc_key: Option<String>,
    pub qywx_am: Option<String>,
    pub qmsg_key: Option<String>,
    pub qq_id: Option<String>,
    pub tg_bot_token: Option<String>,
    pub tg_chat_id: Option<String>,
    pub push_plus_key: Option<String>,
    pub push_plus_topic: Option<String>,
    pub push_plus_channel: Option<String>,
    pub push_plus_webhook: Option<String>,
    pub discord_webhook: Option<String>,
    pub lark_webhook: Option<String>,
    pub lark_secret: Option<String>,

    // Notification templates
    pub qq_template: Option<String>,
    pub tg_template: Option<String>,
    pub wx_template: Option<String>,
    pub sc_template: Option<String>,
    pub discord_template: Option<String>,
    pub lark_template: Option<String>,

    // Avatar
    pub avatar_proxy: Option<String>,
    pub gravatar_str: Option<String>,

    // Markdown
    pub markdown_config: String,
    pub markdown_highlight: Option<String>,
    pub markdown_emoji: Option<String>,
    pub markdown_sub: Option<String>,
    pub markdown_sup: Option<String>,
    pub markdown_tex: String,
    pub markdown_mathjax: String,
    pub markdown_katex: String,

    // OAuth
    pub oauth_url: String,

    // Levels
    pub levels: Option<Vec<i64>>,

    // Like
    pub like_inc_max: i32,

    // Webhook
    pub webhook: Option<String>,

    // IP geolocation
    pub ip2region_db: Option<String>,
    pub ip2region_db_v4: Option<String>,
    pub ip2region_db_v6: Option<String>,

    // Admin
    pub waline_admin_module_asset_url: Option<String>,

    // Detected database type
    pub db_type: DatabaseType,
}

fn parse_boolish(val: &str) -> bool {
    !matches!(val.to_lowercase().as_str(), "0" | "false")
}

fn is_false(val: &str) -> bool {
    matches!(val.to_lowercase().as_str(), "0" | "false")
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, String> {
        let pg_db = env::var("PG_DB")
            .ok()
            .or_else(|| env::var("POSTGRES_DATABASE").ok());
        let pg_host = env::var("PG_HOST")
            .ok()
            .or_else(|| env::var("POSTGRES_HOST").ok());
        let pg_port = env::var("PG_PORT")
            .ok()
            .or_else(|| env::var("POSTGRES_PORT").ok())
            .and_then(|v| v.parse().ok());
        let pg_user = env::var("PG_USER")
            .ok()
            .or_else(|| env::var("POSTGRES_USER").ok());
        let pg_password = env::var("PG_PASSWORD")
            .ok()
            .or_else(|| env::var("POSTGRES_PASSWORD").ok());
        let pg_prefix = env::var("PG_PREFIX")
            .ok()
            .or_else(|| env::var("POSTGRES_PREFIX").ok());
        let pg_ssl = env::var("PG_SSL")
            .ok()
            .or_else(|| env::var("POSTGRES_SSL").ok())
            .map(|v| parse_boolish(&v));
        let postgres_url = env::var("POSTGRES_URL").ok();

        let mysql_db = env::var("MYSQL_DB").ok();
        let mysql_host = env::var("MYSQL_HOST").ok();
        let mysql_port = env::var("MYSQL_PORT").ok().and_then(|v| v.parse().ok());
        let mysql_user = env::var("MYSQL_USER").ok();
        let mysql_password = env::var("MYSQL_PASSWORD").ok();
        let mysql_prefix = env::var("MYSQL_PREFIX").ok();
        let mysql_charset = env::var("MYSQL_CHARSET").ok();
        let mysql_ssl = env::var("MYSQL_SSL").ok().map(|v| parse_boolish(&v));

        let sqlite_path = env::var("SQLITE_PATH")
            .ok()
            .or_else(|| env::var("SQLITE_DB").ok());
        let sqlite_prefix = env::var("SQLITE_PREFIX").ok();

        // Auto-detect database type: PostgreSQL → MySQL → SQLite
        let (db_type, fallback_jwt_key) = if pg_db.is_some() || postgres_url.is_some() {
            (
                DatabaseType::Postgresql,
                pg_password.clone(),
            )
        } else if mysql_db.is_some() {
            (
                DatabaseType::Mysql,
                mysql_password.clone(),
            )
        } else if sqlite_path.is_some() {
            (DatabaseType::Sqlite, None)
        } else {
            return Err(
                "No valid database found. Please set PG_DB/POSTGRES_DATABASE, MYSQL_DB, or SQLITE_PATH environment variables.".to_string()
            );
        };

        let jwt_token = env::var("JWT_TOKEN").ok();
        let jwt_key = jwt_token
            .clone()
            .or(fallback_jwt_key)
            .unwrap_or_default();

        let secure_domains = env::var("SECURE_DOMAINS")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect());

        let disable_useragent = env::var("DISABLE_USERAGENT")
            .ok()
            .map(|v| parse_boolish(&v))
            .unwrap_or(false);

        let disable_region = env::var("DISABLE_REGION")
            .ok()
            .map(|v| parse_boolish(&v))
            .unwrap_or(false);

        let forbidden_words = env::var("FORBIDDEN_WORDS")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let ipqps = env::var("IPQPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let comment_audit = env::var("COMMENT_AUDIT")
            .ok()
            .map(|v| parse_boolish(&v))
            .unwrap_or(false);

        let login = env::var("LOGIN").ok();

        let akismet_key = env::var("AKISMET_KEY").ok();
        let recaptcha_v3_secret = env::var("RECAPTCHA_V3_SECRET").ok();
        let recaptcha_v3_key = env::var("RECAPTCHA_V3_KEY").ok();
        let turnstile_secret = env::var("TURNSTILE_SECRET").ok();
        let turnstile_key = env::var("TURNSTILE_KEY").ok();

        let site_name = env::var("SITE_NAME").ok();
        let site_url = env::var("SITE_URL").ok();
        let server_url = env::var("SERVER_URL").ok();

        let smtp_service = env::var("SMTP_SERVICE").ok();
        let smtp_host = env::var("SMTP_HOST").ok();
        let smtp_port = env::var("SMTP_PORT").ok().and_then(|v| v.parse().ok());
        let smtp_secure = env::var("SMTP_SECURE").ok().map(|v| parse_boolish(&v));
        let smtp_user = env::var("SMTP_USER").ok();
        let smtp_pass = env::var("SMTP_PASS").ok();
        let sender_email = env::var("SENDER_EMAIL").ok();
        let sender_name = env::var("SENDER_NAME").ok();
        let author_email = env::var("AUTHOR_EMAIL").ok();
        let disable_author_notify = env::var("DISABLE_AUTHOR_NOTIFY")
            .ok()
            .map(|v| parse_boolish(&v))
            .unwrap_or(false);

        let avatar_proxy = env::var("AVATAR_PROXY").ok().and_then(|v| {
            if is_false(&v) {
                None
            } else {
                Some(v)
            }
        });
        let gravatar_str = env::var("GRAVATAR_STR").ok();

        let markdown_config = env::var("MARKDOWN_CONFIG").unwrap_or_else(|_| "{}".to_string());
        let markdown_highlight = env::var("MARKDOWN_HIGHLIGHT").ok();
        let markdown_emoji = env::var("MARKDOWN_EMOJI").ok();
        let markdown_sub = env::var("MARKDOWN_SUB").ok();
        let markdown_sup = env::var("MARKDOWN_SUP").ok();
        let markdown_tex = env::var("MARKDOWN_TEX").unwrap_or_else(|_| "mathjax".to_string());
        let markdown_mathjax = env::var("MARKDOWN_MATHJAX").unwrap_or_else(|_| "{}".to_string());
        let markdown_katex = env::var("MARKDOWN_KATEX").unwrap_or_else(|_| "{}".to_string());

        let oauth_url = env::var("OAUTH_URL").unwrap_or_else(|_| "https://oauth.lithub.cc".to_string());

        let levels = env::var("LEVELS")
            .ok()
            .and_then(|v| {
                if is_false(&v) {
                    None
                } else {
                    Some(v.split(',').map(|s| s.trim().parse().ok()).collect::<Option<Vec<_>>>())
                }
            })
            .flatten();

        let like_inc_max = env::var("LIKE_INC_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let webhook = env::var("WEBHOOK").ok();

        let ip2region_db = env::var("IP2REGION_DB").ok();
        let ip2region_db_v4 = env::var("IP2REGION_DB_V4").ok();
        let ip2region_db_v6 = env::var("IP2REGION_DB_V6").ok();

        let waline_admin_module_asset_url = env::var("WALINE_ADMIN_MODULE_ASSET_URL").ok();

        Ok(Self {
            pg_db,
            pg_host,
            pg_port,
            pg_user,
            pg_password,
            pg_prefix,
            pg_ssl,
            postgres_url,
            mysql_db,
            mysql_host,
            mysql_port,
            mysql_user,
            mysql_password,
            mysql_prefix,
            mysql_charset,
            mysql_ssl,
            sqlite_path,
            sqlite_prefix,
            jwt_token,
            jwt_key,
            secure_domains,
            disable_useragent,
            disable_region,
            forbidden_words,
            ipqps,
            comment_audit,
            login,
            akismet_key,
            recaptcha_v3_secret,
            recaptcha_v3_key,
            turnstile_secret,
            turnstile_key,
            site_name,
            site_url,
            server_url,
            smtp_service,
            smtp_host,
            smtp_port,
            smtp_secure,
            smtp_user,
            smtp_pass,
            sender_email,
            sender_name,
            author_email,
            disable_author_notify,
            mail_subject: env::var("MAIL_SUBJECT").ok(),
            mail_template: env::var("MAIL_TEMPLATE").ok(),
            mail_subject_admin: env::var("MAIL_SUBJECT_ADMIN").ok(),
            mail_template_admin: env::var("MAIL_TEMPLATE_ADMIN").ok(),
            sc_key: env::var("SC_KEY").ok(),
            qywx_am: env::var("QYWX_AM").ok(),
            qmsg_key: env::var("QMSG_KEY").ok(),
            qq_id: env::var("QQ_ID").ok(),
            tg_bot_token: env::var("TG_BOT_TOKEN").ok(),
            tg_chat_id: env::var("TG_CHAT_ID").ok(),
            push_plus_key: env::var("PUSH_PLUS_KEY").ok(),
            push_plus_topic: env::var("PUSH_PLUS_TOPIC").ok(),
            push_plus_channel: env::var("PUSH_PLUS_CHANNEL").ok(),
            push_plus_webhook: env::var("PUSH_PLUS_WEBHOOK").ok(),
            discord_webhook: env::var("DISCORD_WEBHOOK").ok(),
            lark_webhook: env::var("LARK_WEBHOOK").ok(),
            lark_secret: env::var("LARK_SECRET").ok(),
            qq_template: env::var("QQ_TEMPLATE").ok(),
            tg_template: env::var("TG_TEMPLATE").ok(),
            wx_template: env::var("WX_TEMPLATE").ok(),
            sc_template: env::var("SC_TEMPLATE").ok(),
            discord_template: env::var("DISCORD_TEMPLATE").ok(),
            lark_template: env::var("LARK_TEMPLATE").ok(),
            avatar_proxy,
            gravatar_str,
            markdown_config,
            markdown_highlight,
            markdown_emoji,
            markdown_sub,
            markdown_sup,
            markdown_tex,
            markdown_mathjax,
            markdown_katex,
            oauth_url,
            levels,
            like_inc_max,
            webhook,
            ip2region_db,
            ip2region_db_v4,
            ip2region_db_v6,
            waline_admin_module_asset_url,
            db_type,
        })
    }

    /// Get the table prefix for the selected database
    pub fn table_prefix(&self) -> &str {
        match self.db_type {
            DatabaseType::Postgresql => self.pg_prefix.as_deref().unwrap_or("wl_"),
            DatabaseType::Mysql => self.mysql_prefix.as_deref().unwrap_or("wl_"),
            DatabaseType::Sqlite => self.sqlite_prefix.as_deref().unwrap_or("wl_"),
        }
    }

    /// Check if emoji is enabled in markdown
    pub fn markdown_emoji_enabled(&self) -> bool {
        self.markdown_emoji
            .as_deref()
            .map(|v| !is_false(v))
            .unwrap_or(true)
    }

    /// Check if subscript is enabled in markdown
    pub fn markdown_sub_enabled(&self) -> bool {
        self.markdown_sub
            .as_deref()
            .map(|v| !is_false(v))
            .unwrap_or(true)
    }

    /// Check if superscript is enabled in markdown
    pub fn markdown_sup_enabled(&self) -> bool {
        self.markdown_sup
            .as_deref()
            .map(|v| !is_false(v))
            .unwrap_or(true)
    }

    /// Get the tex rendering mode
    pub fn markdown_tex_mode(&self) -> Option<&str> {
        if is_false(&self.markdown_tex) {
            None
        } else {
            Some(&self.markdown_tex)
        }
    }

    /// Check if code highlighting is enabled
    pub fn markdown_highlight_enabled(&self) -> bool {
        self.markdown_highlight
            .as_deref()
            .map(|v| !is_false(v))
            .unwrap_or(true)
    }

    /// Check if forced login is required
    pub fn is_force_login(&self) -> bool {
        self.login.as_deref() == Some("force")
    }

    /// Check if any non-email notification channel is configured
    pub fn has_non_email_channel(&self) -> bool {
        self.sc_key.is_some()
            || self.qywx_am.is_some()
            || self.qmsg_key.is_some()
            || self.tg_bot_token.is_some()
            || self.push_plus_key.is_some()
            || self.discord_webhook.is_some()
            || self.lark_webhook.is_some()
    }

    /// Check if SMTP/email is configured
    pub fn has_smtp(&self) -> bool {
        self.smtp_host.is_some() || self.smtp_service.is_some()
    }
}
