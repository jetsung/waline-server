use waline_core::config::Config;

/// Check if the request's origin/referrer is allowed by secure domains config
pub fn check_secure_domains(referer: Option<&str>, origin: Option<&str>, config: &Config) -> bool {
    let secure_domains = match &config.secure_domains {
        Some(domains) if !domains.is_empty() => domains,
        _ => return true, // No secure domains configured = allow all
    };

    let url_to_check = origin.or(referer).unwrap_or("");

    if url_to_check.is_empty() {
        return true; // No origin/referer = allow (some clients don't send these)
    }

    // Extract host from URL
    let host = extract_host(url_to_check);

    for domain in secure_domains {
        if domain.starts_with('/') && domain.ends_with('/') {
            // Regex pattern: /pattern/
            let pattern = &domain[1..domain.len()-1];
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&host) {
                    return true;
                }
            }
        } else {
            // Exact match or suffix match
            if host == *domain || host.ends_with(&format!(".{domain}")) {
                return true;
            }
        }
    }

    false
}

fn extract_host(url: &str) -> String {
    // Simple host extraction: remove protocol and path
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    without_protocol
        .split('/')
        .next()
        .unwrap_or(without_protocol)
        .split(':')
        .next()
        .unwrap_or(without_protocol)
        .to_string()
}

/// Check if comment audit mode is enabled
pub fn is_audit_mode(config: &Config) -> bool {
    config.comment_audit
}

/// Check if forced login is required
pub fn is_force_login(config: &Config) -> bool {
    config.is_force_login()
}

/// Calculate user level based on comment count and LEVELS config
pub fn calculate_level(comment_count: i64, config: &Config) -> Option<i32> {
    config.levels.as_ref()?;
    let levels = config.levels.as_ref().unwrap();
    if levels.is_empty() {
        return None;
    }
    let mut level = 0i32;
    for (i, threshold) in levels.iter().enumerate() {
        if comment_count >= *threshold {
            level = i as i32 + 1;
        }
    }
    Some(level)
}

/// Calculate random like increment
pub fn calculate_like_increment(config: &Config) -> i32 {
    if config.like_inc_max <= 1 {
        return 1;
    }
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=config.like_inc_max)
}
