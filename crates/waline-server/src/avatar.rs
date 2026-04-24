use md5::Digest;
use waline_core::config::Config;

/// Compute MD5 hash
fn md5_hex(input: &[u8]) -> String {
    let mut hasher = md5::Context::new();
    hasher.consume(input);
    format!("{:x}", hasher.compute())
}

/// Generate Gravatar URL from email
pub fn gravatar_url(email: &str, config: &Config) -> String {
    let email_lower = email.trim().to_lowercase();
    let hash = md5_hex(email_lower.as_bytes());
    let template = config.gravatar_str.as_deref().unwrap_or("https://gravatar.com/avatar/{{hash}}");

    let url = template.replace("{{hash}}", &hash);

    // Apply avatar proxy
    if let Some(ref proxy) = config.avatar_proxy {
        format!("{proxy}/{url}")
    } else {
        url
    }
}

/// Generate QQ avatar URL from QQ number
pub fn qq_avatar_url(qq: &str) -> String {
    format!("https://q1.qlogo.cn/g?b=qq&nk={qq}&s=100")
}

/// Get avatar URL for a user, checking QQ first then falling back to Gravatar
pub fn get_avatar(email: Option<&str>, qq: Option<&str>, config: &Config) -> Option<String> {
    if let Some(qq_id) = qq {
        if !qq_id.is_empty() {
            let url = qq_avatar_url(qq_id);
            return if let Some(ref proxy) = config.avatar_proxy {
                Some(format!("{proxy}/{url}"))
            } else {
                Some(url)
            };
        }
    }
    email.map(|e| gravatar_url(e, config))
}
