use async_trait::async_trait;
use thiserror::Error;
use tracing::warn;

mod ip2region_core;
mod ip2region;

pub use ip2region::Ip2RegionGeoIp;
pub use ip2region_core::CachePolicy;

/// GeoIP 错误类型
#[derive(Error, Debug)]
pub enum GeoIpError {
    #[error("Database file not found: {0}")]
    DatabaseNotFound(String),

    #[error("Failed to initialize GeoIP database: {0}")]
    InitializationError(String),

    #[error("Failed to lookup IP address: {0}")]
    LookupError(String),

    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// GeoIP 信息结构体
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeoIpInfo {
    /// 国家
    pub country: String,
    /// 省份
    pub province: String,
    /// 城市
    pub city: String,
    /// ISP (互联网服务提供商)
    pub isp: String,
}

impl GeoIpInfo {
    /// 创建新的 GeoIpInfo 实例
    pub fn new(country: String, province: String, city: String, isp: String) -> Self {
        Self {
            country,
            province,
            city,
            isp,
        }
    }

    /// 创建空的 GeoIpInfo 实例
    pub fn empty() -> Self {
        Self::default()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.country.is_empty()
            && self.province.is_empty()
            && self.city.is_empty()
            && self.isp.is_empty()
    }
}

/// GeoIP trait 定义
#[async_trait]
pub trait GeoIp: Send + Sync {
    /// 根据 IP 地址查询地理位置信息
    ///
    /// # Arguments
    ///
    /// * `ip` - IP 地址字符串 (支持 IPv4 和 IPv6)
    ///
    /// # Returns
    ///
    /// * `Ok(GeoIpInfo)` - 查询成功，返回地理位置信息
    /// * `Err(GeoIpError)` - 查询失败，返回错误信息
    async fn lookup(&self, ip: &str) -> Result<GeoIpInfo, GeoIpError>;

    /// 根据 IP 地址查询地理位置信息，失败时返回空信息而不是错误
    /// 这个方法用于降级处理，确保 GeoIP 查询失败不会影响核心功能
    ///
    /// # Arguments
    ///
    /// * `ip` - IP 地址字符串 (支持 IPv4 和 IPv6)
    ///
    /// # Returns
    ///
    /// * `GeoIpInfo` - 查询成功返回地理位置信息，失败返回空信息
    async fn lookup_or_empty(&self, ip: &str) -> GeoIpInfo {
        match self.lookup(ip).await {
            Ok(info) => info,
            Err(e) => {
                warn!(
                    "GeoIP lookup failed for IP {}, returning empty info: {}",
                    ip, e
                );
                GeoIpInfo::empty()
            }
        }
    }
}

/// NullGeoIp 实现 - 当 GeoIP 功能禁用或数据库不可用时使用
/// 所有查询都返回空的地理位置信息，不会产生错误
pub struct NullGeoIp;

impl NullGeoIp {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullGeoIp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GeoIp for NullGeoIp {
    async fn lookup(&self, _ip: &str) -> Result<GeoIpInfo, GeoIpError> {
        Ok(GeoIpInfo::empty())
    }

    async fn lookup_or_empty(&self, _ip: &str) -> GeoIpInfo {
        GeoIpInfo::empty()
    }
}

/// 创建 GeoIP 实例的辅助函数，支持优雅降级
///
/// 当数据库文件不存在时，记录警告并返回 NullGeoIp 实例，
/// 确保 GeoIP 功能不可用不会影响系统的核心功能
///
/// # Arguments
///
/// * `db_path` - ip2region 数据库文件路径
/// * `cache_policy` - 缓存策略
///
/// # Returns
///
/// * `Box<dyn GeoIp>` - GeoIP 实例（可能是 Ip2RegionGeoIp 或 NullGeoIp）
pub fn create_geoip_with_fallback<P: AsRef<std::path::Path>>(
    db_path: P,
    cache_policy: CachePolicy,
) -> Box<dyn GeoIp> {
    match Ip2RegionGeoIp::new(db_path.as_ref(), cache_policy) {
        Ok(geoip) => Box::new(geoip),
        Err(e) => {
            warn!(
                "Failed to initialize GeoIP database at {}: {}. GeoIP functionality will be disabled.",
                db_path.as_ref().display(),
                e
            );
            Box::new(NullGeoIp::new())
        }
    }
}

/// 根据配置创建 GeoIP 实例
///
/// # Arguments
///
/// * `config` - GeoIP 配置
///
/// # Returns
///
/// * `Option<Arc<dyn GeoIp>>` - GeoIP 实例（如果启用）
pub async fn create_geoip(
    config: &crate::config::GeoIpConfig,
) -> Option<std::sync::Arc<dyn GeoIp>> {
    use crate::config::GeoIpType;

    if !config.enabled {
        return None;
    }

    match config.geoip_type {
        GeoIpType::Ip2region => {
            if let Some(ip2region_config) = &config.ip2region {
                let cache_policy = match ip2region_config.mode.as_str() {
                    "vector" => CachePolicy::VectorIndex,
                    "full" | "memory" => CachePolicy::FullMemory,
                    _ => CachePolicy::NoCache,
                };

                let geoip = create_geoip_with_fallback(&ip2region_config.path, cache_policy);
                Some(std::sync::Arc::from(geoip))
            } else {
                warn!("GeoIP enabled but no ip2region configuration provided");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geoip_info_new() {
        let info = GeoIpInfo::new(
            "中国".to_string(),
            "浙江省".to_string(),
            "杭州市".to_string(),
            "电信".to_string(),
        );

        assert_eq!(info.country, "中国");
        assert_eq!(info.province, "浙江省");
        assert_eq!(info.city, "杭州市");
        assert_eq!(info.isp, "电信");
        assert!(!info.is_empty());
    }

    #[test]
    fn test_geoip_info_empty() {
        let info = GeoIpInfo::empty();
        assert!(info.is_empty());
        assert_eq!(info.country, "");
        assert_eq!(info.province, "");
        assert_eq!(info.city, "");
        assert_eq!(info.isp, "");
    }

    #[test]
    fn test_geoip_info_default() {
        let info = GeoIpInfo::default();
        assert!(info.is_empty());
    }

    #[test]
    fn test_geoip_error_display() {
        let err = GeoIpError::DatabaseNotFound("/path/to/db".to_string());
        assert_eq!(err.to_string(), "Database file not found: /path/to/db");

        let err = GeoIpError::InvalidIpAddress("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid IP address: invalid");
    }

    #[tokio::test]
    async fn test_null_geoip() {
        let geoip = NullGeoIp::new();

        // Test lookup returns empty info
        let result = geoip.lookup("8.8.8.8").await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.is_empty());

        // Test lookup_or_empty returns empty info
        let info = geoip.lookup_or_empty("1.1.1.1").await;
        assert!(info.is_empty());
    }

    #[tokio::test]
    async fn test_null_geoip_default() {
        let geoip = NullGeoIp;
        let info = geoip.lookup_or_empty("192.168.1.1").await;
        assert!(info.is_empty());
    }

    #[tokio::test]
    async fn test_create_geoip_with_fallback_nonexistent_file() {
        // Test with non-existent file - should return NullGeoIp
        let geoip = create_geoip_with_fallback("/nonexistent/path/to/db.xdb", CachePolicy::NoCache);

        // Should not panic and should return empty info
        let info = geoip.lookup_or_empty("8.8.8.8").await;
        assert!(info.is_empty());
    }

    #[tokio::test]
    async fn test_create_geoip_with_fallback_valid_file() {
        // Try to find a valid database file
        let db_path = vec![
            std::path::PathBuf::from("../data/ip2region.xdb"),
            std::path::PathBuf::from("./data/ip2region.xdb"),
            std::path::PathBuf::from("../../data/ip2region.xdb"),
        ]
        .into_iter()
        .find(|p| p.exists());

        if let Some(path) = db_path {
            let geoip = create_geoip_with_fallback(&path, CachePolicy::VectorIndex);

            // Should successfully lookup
            let result = geoip.lookup("8.8.8.8").await;
            assert!(result.is_ok());
            let info = result.unwrap();
            assert!(!info.is_empty());
        } else {
            println!("Skipping test: ip2region.xdb not found");
        }
    }
}
