/// Synchronous IP-to-region lookup.
/// This is a compatibility wrapper for the async geoip module.
/// Returns region string like "中国 广东 深圳" or empty string.
pub fn lookup(ip: &str) -> String {
    if ip.is_empty() {
        return String::new();
    }

    // Use tokio runtime to call async geoip
    // This is a temporary solution; ideally the caller should use async lookup
    let rt = tokio::runtime::Handle::try_current();
    match rt {
        Ok(handle) => {
            // We're in a tokio context, use block_in_place
            tokio::task::block_in_place(|| {
                handle.block_on(async { lookup_async(ip).await })
            })
        }
        Err(_) => {
            // Not in tokio context, create a new runtime
            tokio::runtime::Runtime::new()
                .map(|rt| rt.block_on(async { lookup_async(ip).await }))
                .unwrap_or_default()
        }
    }
}

/// Async IP-to-region lookup using the global geoip instance.
/// Returns region string like "中国 广东 深圳" or empty string.
pub async fn lookup_async(ip: &str) -> String {
    if ip.is_empty() {
        return String::new();
    }

    // Get geoip instance from global state
    let geoip = match crate::state::get_geoip() {
        Some(g) => g,
        None => return String::new(),
    };

    let info = geoip.lookup_or_empty(ip).await;

    // Format: "国家 省份 城市"
    let mut parts = Vec::new();
    if !info.country.is_empty() && info.country != "0" {
        parts.push(info.country.as_str());
    }
    if !info.province.is_empty() && info.province != "0" {
        parts.push(info.province.as_str());
    }
    if !info.city.is_empty() && info.city != "0" {
        parts.push(info.city.as_str());
    }

    parts.join(" ")
}
