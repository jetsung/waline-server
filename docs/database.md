# 数据库适配器

Waline Rust 支持 3 种数据库后端，通过 SQLx 实现数据库无关性。

## 支持的数据库

| 数据库 | 连接字符串示例 | 说明 |
|--------|----------------|------|
| SQLite | `sqlite://./data/waline.sqlite` | 轻量级，适合小型部署 |
| MySQL | `mysql://user:pass@host:port/db` | 适合生产环境 |
| PostgreSQL | `postgres://user:pass@host:port/db` | 适合生产环境 |

## 数据库配置

### 环境变量

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `DATABASE_URL` | 数据库连接字符串 | `sqlite://./data/waline.sqlite` (必填) |

### SQLite

```bash
DATABASE_URL=sqlite://./data/waline.sqlite
```

### MySQL

```bash
DATABASE_URL=mysql://root:password@127.0.0.1:3306/waline
```

### PostgreSQL

```bash
DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/waline
```

## 自动建表

首次启动时，服务会根据检测到的数据库类型自动创建所需的表：

- `wl_Comment` — 评论数据
- `wl_Counter` — 文章计数
- `wl_Users` — 用户数据

无需手动执行 SQL。

## 表结构

### wl_Comment

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER/ SERIAL | 主键 |
| user_id | INTEGER | 关联用户 ID |
| comment | TEXT | 评论内容 |
| insertedAt | DATETIME | 创建时间 |
| ip | VARCHAR(100) | IP 地址 |
| link | VARCHAR(255) | 用户链接 |
| mail | VARCHAR(255) | 用户邮箱 |
| nick | VARCHAR(255) | 用户昵称 |
| pid | INTEGER | 父评论 ID |
| rid | INTEGER | 根评论 ID |
| sticky | BOOLEAN | 置顶 |
| status | VARCHAR(50) | 状态: approved/waiting/spam |
| like | INTEGER | 点赞数 |
| ua | TEXT | User Agent |
| url | VARCHAR(255) | 文章路径 |
| createdAt | DATETIME | 创建时间 |
| updatedAt | DATETIME | 更新时间 |

### wl_Users

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER / SERIAL | 主键 |
| display_name | VARCHAR(255) | 显示名称 |
| email | VARCHAR(255) | 邮箱 (唯一) |
| password | VARCHAR(255) | 密码哈希 |
| type | VARCHAR(50) | 类型: administrator/guest/banned |
| label | VARCHAR(255) | 用户标签 |
| url | VARCHAR(255) | 个人链接 |
| avatar | VARCHAR(255) | 头像 URL |
| 2fa | VARCHAR(32) | 双因素认证密钥 |
| github | VARCHAR(255) | GitHub OAuth ID |
| twitter | VARCHAR(255) | Twitter OAuth ID |
| facebook | VARCHAR(255) | Facebook OAuth ID |
| google | VARCHAR(255) | Google OAuth ID |
| weibo | VARCHAR(255) | 微博 OAuth ID |
| qq | VARCHAR(255) | QQ OAuth ID |
| oidc | VARCHAR(255) | OIDC OAuth ID |
| huawei | VARCHAR(255) | 华为 OAuth ID |
| createdAt | DATETIME | 创建时间 |
| updatedAt | DATETIME | 更新时间 |

### wl_Counter

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER / SERIAL | 主键 |
| url | VARCHAR(255) | 文章路径 |
| time | INTEGER | 访问次数 |
| reaction0 - reaction8 | INTEGER | 反应计数 |
| createdAt | DATETIME | 创建时间 |
| updatedAt | DATETIME | 更新时间 |

## 数据迁移

由于表结构与原版 Waline 完全兼容，可以轻松迁移：

1. 从原版数据库导出数据
2. 导入到新数据库
3. 启动 waline-rust 服务

密码哈希完全兼容 phpass (`$P$`/`$H$`) 和 bcrypt (`$2b$`/`$2a$`/`$2y$`)，用户无需重置密码。

## 与原版 Waline 的差异

原版 Waline 额外支持：
- MongoDB
- LeanCloud
- TiDB
- CloudBase
- GitHub 存储

Rust 版本仅支持 PostgreSQL、MySQL、SQLite，这三种数据库已覆盖绝大多数使用场景。
