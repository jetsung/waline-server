use waline_core::config::Config;

/// Resolve IP address to geographic location string
/// Uses ip2region or returns empty if disabled or not available
pub fn resolve_ip(ip: &str, config: &Config) -> Option<String> {
    if config.disable_region {
        return None;
    }
    // TODO: integrate ip2region library for actual lookup
    // For now, return None (feature not yet integrated)
    let _ = ip;
    None
}

/// Parse User-Agent string to extract browser and OS info
pub fn parse_ua(ua: &str, config: &Config) -> Option<(String, String)> {
    if config.disable_useragent {
        return None;
    }
    // Simple UA parsing - TODO: integrate woothee for proper parsing
    let browser = parse_browser(ua);
    let os = parse_os(ua);
    Some((browser, os))
}

fn parse_browser(ua: &str) -> String {
    if ua.contains("Edg/") {
        "Edge".to_string()
    } else if ua.contains("Chrome/") {
        "Chrome".to_string()
    } else if ua.contains("Firefox/") {
        "Firefox".to_string()
    } else if ua.contains("Safari/") && !ua.contains("Chrome/") {
        "Safari".to_string()
    } else if ua.contains("MicroMessenger/") {
        "WeChat".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn parse_os(ua: &str) -> String {
    if ua.contains("Windows") {
        "Windows".to_string()
    } else if ua.contains("Mac OS") {
        "macOS".to_string()
    } else if ua.contains("Android") {
        "Android".to_string()
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS".to_string()
    } else if ua.contains("Linux") {
        "Linux".to_string()
    } else {
        "Unknown".to_string()
    }
}
