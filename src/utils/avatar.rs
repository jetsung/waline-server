/// Compute Gravatar URL for an email address.
pub fn gravatar_url(email: &str, default_str: Option<&str>) -> String {
    let hash = md5_hex(email.trim().to_lowercase().as_str());
    let default = default_str.unwrap_or("mp");
    format!("https://www.gravatar.com/avatar/{hash}?d={default}&s=80")
}

/// Apply avatar proxy if configured.
pub fn proxied_avatar(url: &str, proxy: Option<&str>) -> String {
    match proxy {
        Some(p) if !url.contains(p) => {
            format!("{}?url={}", p, urlencoding::encode(url))
        }
        _ => url.to_string(),
    }
}

fn md5_hex(s: &str) -> String {
    let digest = md5::compute(s.as_bytes());
    format!("{:x}", digest)
}
