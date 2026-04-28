use serde::Deserialize;
use std::env;
use std::fs;

fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8360
}
fn default_ipqps() -> u64 {
    60
}
fn default_like_inc_max() -> u32 {
    1
}
fn default_oauth_url() -> String {
    "https://oauth.lithub.cc".to_string()
}
fn default_admin_asset_url() -> String {
    "//unpkg.com/@waline/admin".to_string()
}
fn default_false() -> bool {
    false
}
fn default_markdown_tex() -> String {
    "mathjax".to_string()
}

fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(vec![]);
    }
    Ok(s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,

    // Database
    pub database_url: String,

    // Auth
    pub jwt_token: Option<String>,
    #[serde(default, deserialize_with = "deserialize_comma_separated")]
    pub secure_domains: Vec<String>,
    pub login: Option<String>,

    // Site
    pub site_name: Option<String>,
    pub site_url: Option<String>,
    pub server_url: Option<String>,

    // Comment
    #[serde(default = "default_false")]
    pub comment_audit: bool,
    #[serde(default, deserialize_with = "deserialize_comma_separated")]
    pub forbidden_words: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_comma_separated")]
    pub disallow_ip_list: Vec<String>,
    #[serde(default = "default_ipqps")]
    pub ipqps: u64,
    pub levels: Option<String>,
    #[serde(default = "default_like_inc_max")]
    pub like_inc_max: u32,
    pub webhook: Option<String>,

    // Anti-spam
    pub akismet_key: Option<String>,
    pub recaptcha_v3_secret: Option<String>,
    pub recaptcha_v3_key: Option<String>,
    pub turnstile_secret: Option<String>,
    pub turnstile_key: Option<String>,

    // User tracking
    #[serde(default = "default_false")]
    pub disable_useragent: bool,
    #[serde(default = "default_false")]
    pub disable_region: bool,

    // IP region
    pub ip2region_db: Option<String>,
    pub ip2region_db_v4: Option<String>,
    pub ip2region_db_v6: Option<String>,

    // SMTP
    pub smtp_service: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_secure: Option<bool>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub sender_email: Option<String>,
    pub sender_name: Option<String>,
    pub author_email: Option<String>,
    #[serde(default = "default_false")]
    pub disable_author_notify: bool,

    // Mail templates
    pub mail_subject: Option<String>,
    pub mail_template: Option<String>,
    pub mail_subject_admin: Option<String>,
    pub mail_template_admin: Option<String>,

    // Notification channels
    pub sc_key: Option<String>,
    pub sc_template: Option<String>,
    pub qywx_am: Option<String>,
    pub wx_template: Option<String>,
    pub qmsg_key: Option<String>,
    pub qq_id: Option<String>,
    pub qq_template: Option<String>,
    pub tg_bot_token: Option<String>,
    pub tg_chat_id: Option<String>,
    pub tg_template: Option<String>,
    pub push_plus_key: Option<String>,
    pub push_plus_topic: Option<String>,
    pub push_plus_channel: Option<String>,
    pub push_plus_webhook: Option<String>,
    pub discord_webhook: Option<String>,
    pub discord_template: Option<String>,
    pub lark_webhook: Option<String>,
    pub lark_secret: Option<String>,
    pub lark_template: Option<String>,

    // Avatar
    pub avatar_proxy: Option<String>,
    pub gravatar_str: Option<String>,

    // Markdown
    pub markdown_highlight: Option<String>,
    pub markdown_emoji: Option<String>,
    pub markdown_sub: Option<String>,
    pub markdown_sup: Option<String>,
    #[serde(default = "default_markdown_tex")]
    pub markdown_tex: String,

    // OAuth
    #[serde(default = "default_oauth_url")]
    pub oauth_url: String,

    // Admin UI
    #[serde(default = "default_admin_asset_url")]
    pub waline_admin_module_asset_url: String,

    #[serde(default = "default_false")]
    pub debug: bool,
}

/// GeoIP configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GeoIpConfig {
    /// Enable GeoIP lookup
    #[serde(default = "default_false")]
    pub enabled: bool,
    /// GeoIP type (currently only "ip2region" supported)
    #[serde(default = "default_geoip_type")]
    pub geoip_type: GeoIpType,
    /// ip2region configuration
    pub ip2region: Option<Ip2RegionConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum GeoIpType {
    Ip2region,
}

fn default_geoip_type() -> GeoIpType {
    GeoIpType::Ip2region
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ip2RegionConfig {
    /// Database file path
    pub path: String,
    /// Cache mode: "vector" (default), "full"/"memory", or "none"
    #[serde(default = "default_ip2region_mode")]
    pub mode: String,
}

fn default_ip2region_mode() -> String {
    "vector".to_string()
}

impl Config {
    /// Build GeoIpConfig from environment variables
    pub fn geoip_config(&self) -> GeoIpConfig {
        // Priority: IP2REGION_DB_V4 > IP2REGION_DB
        let db_path = self.ip2region_db_v4.as_ref()
            .or(self.ip2region_db.as_ref());

        match db_path {
            Some(path) => GeoIpConfig {
                enabled: !self.disable_region,
                geoip_type: GeoIpType::Ip2region,
                ip2region: Some(Ip2RegionConfig {
                    path: path.clone(),
                    mode: "vector".to_string(),
                }),
            },
            None => GeoIpConfig {
                enabled: false,
                geoip_type: GeoIpType::Ip2region,
                ip2region: None,
            },
        }
    }

    pub fn load() -> Result<Self, envy::Error> {
        dotenvy::dotenv_override().ok();
        
        let mut config = if let Ok(content) = fs::read_to_string("config.toml") {
            toml::from_str::<Config>(&content).map_err(|e| envy::Error::Custom(e.to_string()))?
        } else {
            envy::from_env::<Config>()?
        };

        if let Ok(val) = env::var("DATABASE_URL") { config.database_url = val; }
        if let Ok(val) = env::var("HOST") { config.host = val; }
        if let Ok(val) = env::var("PORT") { if let Ok(p) = val.parse() { config.port = p; } }
        if let Ok(val) = env::var("DEBUG") { config.debug = val == "true"; }
        
        Ok(config)
    }

    pub fn jwt_secret(&self) -> String {
        self.jwt_token.clone().unwrap_or_default()
    }

    pub fn is_login_force(&self) -> bool {
        self.login.as_deref() == Some("force")
    }

    pub fn has_smtp(&self) -> bool {
        self.smtp_host.is_some() || self.smtp_service.is_some()
    }

    pub fn markdown_highlight_enabled(&self) -> bool {
        self.markdown_highlight.as_deref() != Some("false")
    }

    pub fn markdown_emoji_enabled(&self) -> bool {
        self.markdown_emoji.as_deref() != Some("false")
    }

    pub fn markdown_sub_enabled(&self) -> bool {
        self.markdown_sub.as_deref() != Some("false")
    }

    pub fn markdown_sup_enabled(&self) -> bool {
        self.markdown_sup.as_deref() != Some("false")
    }

    pub fn avatar_proxy_url(&self) -> Option<&str> {
        match self.avatar_proxy.as_deref() {
            Some("false") | None => None,
            Some(url) => Some(url),
        }
    }
}
