/// Parse UA string into (browser, os) strings.
pub fn parse(ua: &str) -> (String, String) {
    if ua.is_empty() {
        return (String::new(), String::new());
    }
    let browser = parse_browser(ua);
    let os = parse_os(ua);
    (browser, os)
}

fn parse_browser(ua: &str) -> String {
    let ua_lower = ua.to_lowercase();
    if ua_lower.contains("edg/") || ua_lower.contains("edge/") {
        extract_version(ua, "Edg").map(|v| format!("Edge {v}")).unwrap_or_else(|| "Edge".to_string())
    } else if ua_lower.contains("chrome/") {
        extract_version(ua, "Chrome").map(|v| format!("Chrome {v}")).unwrap_or_else(|| "Chrome".to_string())
    } else if ua_lower.contains("firefox/") {
        extract_version(ua, "Firefox").map(|v| format!("Firefox {v}")).unwrap_or_else(|| "Firefox".to_string())
    } else if ua_lower.contains("safari/") {
        extract_version(ua, "Version").map(|v| format!("Safari {v}")).unwrap_or_else(|| "Safari".to_string())
    } else if ua_lower.contains("opr/") || ua_lower.contains("opera/") {
        "Opera".to_string()
    } else {
        String::new()
    }
}

fn parse_os(ua: &str) -> String {
    let ua_lower = ua.to_lowercase();
    if ua_lower.contains("windows nt") {
        let ver = if ua_lower.contains("windows nt 10") { "10" }
            else if ua_lower.contains("windows nt 6.3") { "8.1" }
            else if ua_lower.contains("windows nt 6.2") { "8" }
            else if ua_lower.contains("windows nt 6.1") { "7" }
            else { "" };
        if ver.is_empty() { "Windows".to_string() } else { format!("Windows {ver}") }
    } else if ua_lower.contains("android") {
        extract_version(ua, "Android").map(|v| format!("Android {v}")).unwrap_or_else(|| "Android".to_string())
    } else if ua_lower.contains("iphone") || ua_lower.contains("ipad") {
        "iOS".to_string()
    } else if ua_lower.contains("mac os x") {
        "macOS".to_string()
    } else if ua_lower.contains("linux") {
        "Linux".to_string()
    } else {
        String::new()
    }
}

fn extract_version(ua: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}/");
    let start = ua.find(&pattern)? + pattern.len();
    let rest = &ua[start..];
    let end = rest.find(|c: char| c == ' ' || c == ';' || c == ')').unwrap_or(rest.len());
    let ver = &rest[..end];
    // Return only major.minor
    let parts: Vec<&str> = ver.splitn(3, '.').collect();
    Some(parts[..parts.len().min(2)].join("."))
}
