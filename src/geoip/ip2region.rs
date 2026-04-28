use super::{GeoIp, GeoIpError, GeoIpInfo};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::geoip::ip2region_core::{CachePolicy, Searcher};

/// Ip2Region GeoIP 实现
/// 使用 ip2region-rs crate 支持 ip2region 数据库
///
/// # IPv4 和 IPv6 支持
///
/// ip2region 数据库分为 IPv4 和 IPv6 两个版本：
/// - IPv4 数据库：支持 IPv4 地址查询
/// - IPv6 数据库：支持 IPv6 地址查询
///
/// 需要根据实际需求选择对应版本的数据库文件。
/// 如果需要同时支持 IPv4 和 IPv6，需要创建两个 Ip2RegionGeoIp 实例，
/// 分别加载不同版本的数据库文件。
///
/// # 缓存策略
///
/// 支持三种缓存策略：
/// - `CachePolicy::NoCache`: 不使用缓存，每次查询都从文件读取
/// - `CachePolicy::VectorIndex`: 缓存向量索引（推荐，平衡性能和内存）
/// - `CachePolicy::FullMemory`: 将整个数据库加载到内存（最快，但占用内存较大）
pub struct Ip2RegionGeoIp {
    searcher: Arc<Mutex<Searcher>>,
    db_path: String,
}

impl Ip2RegionGeoIp {
    /// 创建新的 Ip2RegionGeoIp 实例
    ///
    /// # Arguments
    ///
    /// * `db_path` - ip2region 数据库文件路径
    /// * `cache_policy` - 缓存策略 (NoCache, VectorIndex, FullMemory)
    ///
    /// # Returns
    ///
    /// * `Ok(Ip2RegionGeoIp)` - 创建成功
    /// * `Err(GeoIpError)` - 创建失败
    pub fn new<P: AsRef<Path>>(db_path: P, cache_policy: CachePolicy) -> Result<Self, GeoIpError> {
        let path = db_path.as_ref();

        // 检查数据库文件是否存在
        if !path.exists() {
            return Err(GeoIpError::DatabaseNotFound(path.display().to_string()));
        }

        // 创建 Searcher
        let searcher =
            Searcher::new(path.to_str().unwrap().to_string(), cache_policy).map_err(|e| {
                GeoIpError::InitializationError(format!(
                    "Failed to create searcher for {}: {}",
                    path.display(),
                    e
                ))
            })?;

        debug!(
            "Ip2Region GeoIP initialized with database: {} (cache_policy: {:?})",
            path.display(),
            cache_policy
        );

        Ok(Self {
            searcher: Arc::new(Mutex::new(searcher)),
            db_path: path.display().to_string(),
        })
    }

    /// 创建新的 Ip2RegionGeoIp 实例，使用默认的 VectorIndex 缓存策略
    ///
    /// # Arguments
    ///
    /// * `db_path` - ip2region 数据库文件路径
    ///
    /// # Returns
    ///
    /// * `Ok(Ip2RegionGeoIp)` - 创建成功
    /// * `Err(GeoIpError)` - 创建失败
    pub fn with_default_cache<P: AsRef<Path>>(db_path: P) -> Result<Self, GeoIpError> {
        Self::new(db_path, CachePolicy::VectorIndex)
    }

    /// 解析 ip2region 返回的信息
    /// ip2region 返回格式: 国家|省份|城市|ISP (4个字段)
    /// 例如:
    /// - 中国|浙江省|杭州市|电信
    /// - 日本|东京都|千代田区|专线用户
    fn parse_region_string(region: &str) -> GeoIpInfo {
        let parts: Vec<&str> = region.split('|').collect();

        let country = parts.first().unwrap_or(&"").trim().to_string();
        let province = parts.get(1).unwrap_or(&"").trim().to_string();
        let city = parts.get(2).unwrap_or(&"").trim().to_string();
        let isp = parts.get(3).unwrap_or(&"").trim().to_string();

        GeoIpInfo::new(country, province, city, isp)
    }
}

#[async_trait]
impl GeoIp for Ip2RegionGeoIp {
    async fn lookup(&self, ip: &str) -> Result<GeoIpInfo, GeoIpError> {
        // 验证 IP 地址格式
        if ip.is_empty() {
            return Err(GeoIpError::InvalidIpAddress(
                "IP address is empty".to_string(),
            ));
        }

        // 获取 searcher 锁
        let searcher = self.searcher.lock().await;

        // 查询 IP 地址
        let region = searcher.search(ip).map_err(|e| {
            warn!("Failed to lookup IP {} in {}: {}", ip, self.db_path, e);
            GeoIpError::LookupError(format!("Failed to search IP {}: {}", ip, e))
        })?;

        debug!("IP {} lookup result: {}", ip, region);

        // 解析结果
        let geoip_info = Self::parse_region_string(&region);

        Ok(geoip_info)
    }

    async fn lookup_or_empty(&self, ip: &str) -> GeoIpInfo {
        match self.lookup(ip).await {
            Ok(info) => info,
            Err(e) => {
                warn!(
                    "GeoIP lookup failed for IP {} in database {}, returning empty info: {}",
                    ip, self.db_path, e
                );
                GeoIpInfo::empty()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_region_string() {
        // 测试标准4个字段格式（中国）
        let region = "中国|浙江省|杭州市|电信";
        let geoip_info = Ip2RegionGeoIp::parse_region_string(region);
        assert_eq!(geoip_info.country, "中国");
        assert_eq!(geoip_info.province, "浙江省");
        assert_eq!(geoip_info.city, "杭州市");
        assert_eq!(geoip_info.isp, "电信");

        // 测试4个字段格式（日本）
        let region = "日本|东京都|千代田区|专线用户";
        let geoip_info = Ip2RegionGeoIp::parse_region_string(region);
        assert_eq!(geoip_info.country, "日本");
        assert_eq!(geoip_info.province, "东京都");
        assert_eq!(geoip_info.city, "千代田区");
        assert_eq!(geoip_info.isp, "专线用户");

        // 测试不完整信息
        let region = "中国|浙江省";
        let geoip_info = Ip2RegionGeoIp::parse_region_string(region);
        assert_eq!(geoip_info.country, "中国");
        assert_eq!(geoip_info.province, "浙江省");
        assert_eq!(geoip_info.city, "");
        assert_eq!(geoip_info.isp, "");
    }

    #[test]
    fn test_new_with_nonexistent_file() {
        let result = Ip2RegionGeoIp::new("/nonexistent/path/to/db.xdb", CachePolicy::VectorIndex);
        assert!(result.is_err());
        match result {
            Err(GeoIpError::DatabaseNotFound(_)) => {}
            _ => panic!("Expected DatabaseNotFound error"),
        }
    }

    #[test]
    fn test_new_with_default_cache() {
        let result = Ip2RegionGeoIp::with_default_cache("/nonexistent/path/to/db.xdb");
        assert!(result.is_err());
        match result {
            Err(GeoIpError::DatabaseNotFound(_)) => {}
            _ => panic!("Expected DatabaseNotFound error"),
        }
    }

    #[test]
    fn test_new_with_different_cache_policies() {
        // Test NoCache
        let result = Ip2RegionGeoIp::new("/nonexistent/path/to/db.xdb", CachePolicy::NoCache);
        assert!(result.is_err());

        // Test VectorIndex
        let result = Ip2RegionGeoIp::new("/nonexistent/path/to/db.xdb", CachePolicy::VectorIndex);
        assert!(result.is_err());

        // Test FullMemory
        let result = Ip2RegionGeoIp::new("/nonexistent/path/to/db.xdb", CachePolicy::FullMemory);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lookup_with_empty_ip() {
        // This test requires a real database file
        // We'll use the integration test database if available
        let db_path = vec![
            PathBuf::from("../data/ip2region.xdb"),
            PathBuf::from("./data/ip2region.xdb"),
            PathBuf::from("../../data/ip2region.xdb"),
        ]
        .into_iter()
        .find(|p| p.exists());

        if let Some(path) = db_path {
            let geoip = Ip2RegionGeoIp::new(&path, CachePolicy::NoCache).unwrap();
            let result = geoip.lookup("").await;

            assert!(result.is_err());
            match result {
                Err(GeoIpError::InvalidIpAddress(_)) => {}
                _ => panic!("Expected InvalidIpAddress error"),
            }
        } else {
            println!("Skipping test: ip2region.xdb not found");
        }
    }

    #[tokio::test]
    async fn test_lookup_or_empty_with_empty_ip() {
        let db_path = vec![
            PathBuf::from("../data/ip2region.xdb"),
            PathBuf::from("./data/ip2region.xdb"),
            PathBuf::from("../../data/ip2region.xdb"),
        ]
        .into_iter()
        .find(|p| p.exists());

        if let Some(path) = db_path {
            let geoip = Ip2RegionGeoIp::new(&path, CachePolicy::NoCache).unwrap();

            // lookup_or_empty should return empty info instead of error
            let info = geoip.lookup_or_empty("").await;
            assert!(info.is_empty());
        } else {
            println!("Skipping test: ip2region.xdb not found");
        }
    }

    #[tokio::test]
    async fn test_lookup_or_empty_with_valid_ip() {
        let db_path = vec![
            PathBuf::from("../data/ip2region.xdb"),
            PathBuf::from("./data/ip2region.xdb"),
            PathBuf::from("../../data/ip2region.xdb"),
        ]
        .into_iter()
        .find(|p| p.exists());

        if let Some(path) = db_path {
            let geoip = Ip2RegionGeoIp::new(&path, CachePolicy::NoCache).unwrap();

            // lookup_or_empty should return valid info for valid IP
            let info = geoip.lookup_or_empty("8.8.8.8").await;
            assert!(!info.is_empty());
        } else {
            println!("Skipping test: ip2region.xdb not found");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper function to get the test database path
    fn get_test_db_path() -> Option<PathBuf> {
        // Try to find the ip2region.xdb file in the data directory
        let paths = vec![
            PathBuf::from("../data/ip2region.xdb"),
            PathBuf::from("./data/ip2region.xdb"),
            PathBuf::from("../../data/ip2region.xdb"),
        ];

        paths.into_iter().find(|p| p.exists())
    }

    #[tokio::test]
    async fn test_lookup_ipv4_address() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        // Test with a known IPv4 address (Google DNS)
        let result = geoip.lookup("8.8.8.8").await;
        assert!(result.is_ok(), "Failed to lookup 8.8.8.8: {:?}", result);

        let info = result.unwrap();
        println!("8.8.8.8 lookup result: {:?}", info);
        assert!(!info.is_empty(), "GeoIP info should not be empty");

        // Test with another IPv4 address (Cloudflare DNS)
        let result = geoip.lookup("1.1.1.1").await;
        assert!(result.is_ok(), "Failed to lookup 1.1.1.1: {:?}", result);

        let info = result.unwrap();
        println!("1.1.1.1 lookup result: {:?}", info);
        assert!(!info.is_empty(), "GeoIP info should not be empty");
    }

    #[tokio::test]
    async fn test_lookup_with_different_cache_policies() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        // Test with NoCache
        let geoip = Ip2RegionGeoIp::new(&db_path, CachePolicy::NoCache).unwrap();
        let result = geoip.lookup("8.8.8.8").await;
        assert!(result.is_ok());
        println!("NoCache result: {:?}", result.unwrap());

        // Test with VectorIndex
        let geoip = Ip2RegionGeoIp::new(&db_path, CachePolicy::VectorIndex).unwrap();
        let result = geoip.lookup("8.8.8.8").await;
        assert!(result.is_ok());
        println!("VectorIndex result: {:?}", result.unwrap());

        // Test with FullMemory
        let geoip = Ip2RegionGeoIp::new(&db_path, CachePolicy::FullMemory).unwrap();
        let result = geoip.lookup("8.8.8.8").await;
        assert!(result.is_ok());
        println!("FullMemory result: {:?}", result.unwrap());
    }

    #[tokio::test]
    async fn test_lookup_multiple_ips() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        let test_ips = vec![
            "8.8.8.8",         // Google DNS
            "1.1.1.1",         // Cloudflare DNS
            "114.114.114.114", // China DNS
            "223.5.5.5",       // Alibaba DNS
        ];

        for ip in test_ips {
            let result = geoip.lookup(ip).await;
            assert!(result.is_ok(), "Failed to lookup {}: {:?}", ip, result);
            let info = result.unwrap();
            println!("{} -> {:?}", ip, info);
        }
    }

    #[tokio::test]
    async fn test_ipv6_lookup() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        // Test with IPv6 address
        // Note: This will fail if the database is IPv4-only
        // To support IPv6, you need to use an IPv6 version of the ip2region database
        let result = geoip.lookup("2001:4860:4860::8888").await;

        match result {
            Ok(info) => {
                println!("IPv6 lookup successful: {:?}", info);
            }
            Err(e) => {
                println!(
                    "IPv6 lookup failed (expected if using IPv4 database): {:?}",
                    e
                );
                // This is expected if we're using an IPv4-only database
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_ip_address() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        // Test with invalid IP addresses
        let invalid_ips = vec!["invalid", "999.999.999.999", "not.an.ip.address"];

        for ip in invalid_ips {
            let result = geoip.lookup(ip).await;
            println!("Lookup {} result: {:?}", ip, result);
            // The result depends on how ip2region handles invalid IPs
            // It may return an error or a default value
        }
    }

    #[tokio::test]
    async fn test_error_handling_with_empty_ip() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        // Test with empty IP - should return error
        let result = geoip.lookup("").await;
        assert!(result.is_err());
        match result {
            Err(GeoIpError::InvalidIpAddress(_)) => {
                println!("Correctly returned InvalidIpAddress error for empty IP");
            }
            _ => panic!("Expected InvalidIpAddress error for empty IP"),
        }

        // Test lookup_or_empty with empty IP - should return empty info
        let info = geoip.lookup_or_empty("").await;
        assert!(info.is_empty());
        println!("lookup_or_empty correctly returned empty info for empty IP");
    }

    #[tokio::test]
    async fn test_graceful_degradation() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = Ip2RegionGeoIp::with_default_cache(&db_path).unwrap();

        // Test that lookup_or_empty never panics and always returns a result
        let test_ips = vec![
            "",                // Empty
            "invalid",         // Invalid format
            "8.8.8.8",         // Valid IPv4
            "999.999.999.999", // Out of range
            "not.an.ip",       // Clearly invalid
        ];

        for ip in test_ips {
            let info = geoip.lookup_or_empty(ip).await;
            println!("lookup_or_empty({}) -> {:?}", ip, info);
            // Should never panic, always return a GeoIpInfo (may be empty)
        }
    }

    #[tokio::test]
    async fn test_concurrent_lookups_with_errors() {
        let db_path = match get_test_db_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: ip2region.xdb not found");
                return;
            }
        };

        let geoip = std::sync::Arc::new(Ip2RegionGeoIp::with_default_cache(&db_path).unwrap());

        // Test concurrent lookups with mix of valid and invalid IPs
        let mut handles = vec![];

        for i in 0..10 {
            let geoip_clone = geoip.clone();
            let handle = tokio::spawn(async move {
                let ip = if i % 2 == 0 {
                    "8.8.8.8" // Valid
                } else {
                    "" // Invalid
                };
                geoip_clone.lookup_or_empty(ip).await
            });
            handles.push(handle);
        }

        // Wait for all lookups to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent lookup should not panic");
        }

        println!("All concurrent lookups completed successfully");
    }

    #[tokio::test]
    async fn test_database_not_found_error() {
        // Test that creating GeoIP with non-existent file returns proper error
        let result = Ip2RegionGeoIp::new("/nonexistent/path/to/db.xdb", CachePolicy::NoCache);

        assert!(result.is_err());
        match result {
            Err(GeoIpError::DatabaseNotFound(path)) => {
                assert!(path.contains("nonexistent"));
                println!("Correctly returned DatabaseNotFound error: {}", path);
            }
            _ => panic!("Expected DatabaseNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_error_types() {
        // Test all error types can be created and displayed
        let errors = vec![
            GeoIpError::DatabaseNotFound("/path/to/db".to_string()),
            GeoIpError::InitializationError("init failed".to_string()),
            GeoIpError::LookupError("lookup failed".to_string()),
            GeoIpError::InvalidIpAddress("invalid".to_string()),
        ];

        for error in errors {
            let error_string = error.to_string();
            println!("Error: {}", error_string);
            assert!(!error_string.is_empty());
        }
    }
}
