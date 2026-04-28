# GeoIP 错误处理文档

## 概述

本文档描述了 GeoIP 模块的错误处理策略和降级机制，确保 GeoIP 功能不可用时不会影响系统的核心功能。

## 需求

本实现满足以下需求：

- **需求 8.5**: WHEN GeoIP 数据库文件不存在 THEN 系统 SHALL 记录警告但不影响核心功能
- **需求 12.4**: WHEN 发生缓存错误 THEN 系统 SHALL 记录警告但不中断服务

## 错误类型

### GeoIpError

```rust
pub enum GeoIpError {
    DatabaseNotFound(String),      // 数据库文件不存在
    InitializationError(String),   // 初始化失败
    LookupError(String),           // 查询失败
    InvalidIpAddress(String),      // 无效的 IP 地址
    IoError(std::io::Error),       // IO 错误
}
```

## 降级机制

### 1. NullGeoIp

当 GeoIP 数据库不可用时，系统使用 `NullGeoIp` 实现，它：

- 所有查询都返回空的地理位置信息
- 永远不会产生错误
- 不会影响系统的核心功能

```rust
let geoip = NullGeoIp::new();
let info = geoip.lookup("8.8.8.8").await; // 返回 Ok(GeoIpInfo::empty())
```

### 2. lookup_or_empty 方法

`GeoIp` trait 提供了 `lookup_or_empty` 方法，用于优雅降级：

```rust
async fn lookup_or_empty(&self, ip: &str) -> GeoIpInfo {
    match self.lookup(ip).await {
        Ok(info) => info,
        Err(e) => {
            warn!("GeoIP lookup failed for IP {}, returning empty info: {}", ip, e);
            GeoIpInfo::empty()
        }
    }
}
```

使用此方法可以确保查询失败时返回空信息而不是错误，不会中断业务流程。

### 3. create_geoip_with_fallback 函数

提供了一个辅助函数，用于创建 GeoIP 实例时自动处理错误：

```rust
pub fn create_geoip_with_fallback<P: AsRef<Path>>(
    db_path: P,
    cache_policy: CachePolicy,
) -> Box<dyn GeoIp> {
    match Ip2RegionGeoIp::new(db_path.as_ref(), cache_policy) {
        Ok(geoip) => Box::new(geoip),
        Err(e) => {
            warn!("Failed to initialize GeoIP database: {}. GeoIP functionality will be disabled.", e);
            Box::new(NullGeoIp::new())
        }
    }
}
```

## 使用示例

### 场景 1: 数据库文件不存在

```rust
// 使用 create_geoip_with_fallback 自动降级
let geoip = create_geoip_with_fallback("/path/to/missing.xdb", CachePolicy::VectorIndex);

// 系统继续正常运行，只是没有地理位置信息
let info = geoip.lookup_or_empty("8.8.8.8").await;
assert!(info.is_empty()); // 返回空信息，不会崩溃
```

### 场景 2: 查询失败时的降级

```rust
let geoip = Ip2RegionGeoIp::with_default_cache("data/ip2region.xdb")?;

// 使用 lookup 方法 - 返回 Result
let result = geoip.lookup("invalid_ip").await;
match result {
    Ok(info) => println!("Found: {:?}", info),
    Err(e) => eprintln!("Lookup failed: {}", e),
}

// 使用 lookup_or_empty 方法 - 永远不会失败
let info = geoip.lookup_or_empty("invalid_ip").await;
// 返回空信息，记录警告日志
```

### 场景 3: 在业务逻辑中使用

```rust
// 在记录访问历史时使用 GeoIP
async fn record_access(
    geoip: &dyn GeoIp,
    ip: &str,
    // ... 其他参数
) -> Result<(), ServiceError> {
    // 使用 lookup_or_empty 确保 GeoIP 失败不会影响记录访问历史
    let geo_info = geoip.lookup_or_empty(ip).await;

    // 创建历史记录，即使 geo_info 为空也能正常工作
    let history = History {
        ip_address: ip.to_string(),
        country: geo_info.country,
        region: geo_info.region,
        province: geo_info.province,
        city: geo_info.city,
        isp: geo_info.isp,
        // ... 其他字段
    };

    // 保存历史记录
    save_history(history).await?;

    Ok(())
}
```

## 日志记录

### 警告日志

当 GeoIP 功能不可用或查询失败时，系统会记录警告日志：

```
WARN Failed to initialize GeoIP database at /path/to/db.xdb: Database file not found. GeoIP functionality will be disabled.
WARN GeoIP lookup failed for IP 8.8.8.8, returning empty info: Invalid IP address: empty
```

### 调试日志

成功的查询会记录调试日志：

```
DEBUG Ip2Region GeoIP initialized with database: data/ip2region.xdb (cache_policy: VectorIndex)
DEBUG IP 8.8.8.8 lookup result: 美国|0|0|Level3|
```

## 测试

### 单元测试

- `test_null_geoip`: 测试 NullGeoIp 的基本功能
- `test_lookup_or_empty_with_empty_ip`: 测试空 IP 的降级处理
- `test_create_geoip_with_fallback_nonexistent_file`: 测试不存在文件的降级

### 集成测试

- `test_database_not_found_graceful_degradation`: 测试数据库不存在时的优雅降级
- `test_null_geoip_never_fails`: 测试 NullGeoIp 永远不会失败
- `test_lookup_error_degradation`: 测试查询失败时的降级处理
- `test_error_does_not_affect_subsequent_requests`: 测试错误不影响后续请求
- `test_concurrent_requests_with_errors`: 测试并发请求中的错误处理
- `test_system_continues_without_geoip`: 测试系统在 GeoIP 不可用时继续运行

运行测试：

```bash
# 运行所有 GeoIP 测试
cargo test --package shortener-server geoip

# 运行错误处理集成测试
cargo test --package shortener-server --test geoip_error_handling_test
```

## 最佳实践

### 1. 使用 lookup_or_empty 而不是 lookup

在业务逻辑中，优先使用 `lookup_or_empty` 方法，确保 GeoIP 查询失败不会中断业务流程：

```rust
// ✅ 推荐
let info = geoip.lookup_or_empty(ip).await;

// ❌ 不推荐（除非你需要处理错误）
let info = geoip.lookup(ip).await?;
```

### 2. 使用 create_geoip_with_fallback 创建实例

在初始化 GeoIP 时，使用 `create_geoip_with_fallback` 函数自动处理数据库不存在的情况：

```rust
// ✅ 推荐
let geoip = create_geoip_with_fallback(db_path, CachePolicy::VectorIndex);

// ❌ 不推荐（除非你需要处理初始化错误）
let geoip = Ip2RegionGeoIp::new(db_path, CachePolicy::VectorIndex)?;
```

### 3. 检查 GeoIpInfo 是否为空

在使用 GeoIP 信息时，检查是否为空：

```rust
let info = geoip.lookup_or_empty(ip).await;

if !info.is_empty() {
    println!("Location: {}, {}", info.country, info.city);
} else {
    println!("Location information not available");
}
```

### 4. 配置日志级别

在生产环境中，建议将 GeoIP 模块的日志级别设置为 WARN 或更高，避免过多的调试日志：

```rust
RUST_LOG=shortener_server::geoip=warn
```

## 性能考虑

### 1. 缓存策略

选择合适的缓存策略可以提高性能：

- `CachePolicy::NoCache`: 不使用缓存，每次查询都从文件读取（最慢）
- `CachePolicy::VectorIndex`: 缓存向量索引（推荐，平衡性能和内存）
- `CachePolicy::FullMemory`: 将整个数据库加载到内存（最快，但占用内存较大）

### 2. 并发处理

`Ip2RegionGeoIp` 使用 `Arc<Mutex<Searcher>>` 支持并发查询，但 Mutex 可能成为瓶颈。在高并发场景下，考虑：

- 使用 `CachePolicy::FullMemory` 减少文件 IO
- 创建多个 GeoIP 实例（每个实例一个 Searcher）
- 使用连接池模式

### 3. 错误处理开销

`lookup_or_empty` 方法在查询失败时会记录警告日志，在高频率失败的场景下可能影响性能。建议：

- 在调用前验证 IP 地址格式
- 使用 `lookup` 方法并自行处理错误，避免重复记录日志

## 故障排查

### 问题 1: 数据库文件不存在

**症状**: 启动时看到警告日志 "Failed to initialize GeoIP database"

**解决方案**:
1. 检查数据库文件路径是否正确
2. 确保数据库文件存在且可读
3. 下载 ip2region 数据库文件：https://github.com/lionsoul2014/ip2region

### 问题 2: 查询总是返回空信息

**症状**: 所有 IP 查询都返回空的 GeoIpInfo

**可能原因**:
1. 使用了 NullGeoIp 实例（数据库初始化失败）
2. 数据库文件损坏
3. IP 地址格式不正确

**解决方案**:
1. 检查初始化日志，确认是否使用了 NullGeoIp
2. 重新下载数据库文件
3. 验证 IP 地址格式

### 问题 3: IPv6 查询失败

**症状**: IPv6 地址查询返回错误

**原因**: 使用了 IPv4 版本的数据库

**解决方案**: 下载并使用 IPv6 版本的 ip2region 数据库

## 总结

GeoIP 模块的错误处理机制确保了：

1. **可用性**: 数据库不可用时系统继续正常运行
2. **可观测性**: 通过日志记录所有错误和警告
3. **降级策略**: 自动降级到 NullGeoIp，返回空信息而不是错误
4. **易用性**: 提供 `lookup_or_empty` 和 `create_geoip_with_fallback` 简化使用

这种设计符合"优雅降级"的原则，确保 GeoIP 功能的失败不会影响系统的核心功能。
