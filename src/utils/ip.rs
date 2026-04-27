/// Placeholder IP-to-region lookup.
/// Returns region string like "中国 广东 深圳" or empty string.
pub fn lookup(_ip: &str) -> String {
    // TODO: integrate ip2region xdb when IP2REGION_DB is configured
    String::new()
}
