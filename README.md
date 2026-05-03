# Waline Rust

Waline 评论系统的 Rust 重写版本，使用 Axum + SQLx 构建，提供与原版 Node.js 服务端完全兼容的 REST API。

## 功能特性

- **3 种数据库适配器** — PostgreSQL、MySQL、SQLite（自动检测）
- **8 种通知渠道** — Server酱、企业微信、QQ、Telegram、PushPlus、Discord、飞书/Lark、邮件(SMTP)
- **安全认证** — JWT 令牌、phpass/bcrypt 密码兼容、双因素认证(2FA/TOTP)、OAuth2 登录
- **反垃圾** — Akismet、违禁词过滤、IP 频率限制、reCAPTCHA v3、Cloudflare Turnstile
- **Markdown 渲染** — GFM 语法、代码高亮(syntect)、Emoji 短代码、LaTeX 数学公式、XSS 过滤
- **完整 API** — 与原版 Waline 完全兼容的 REST API，可直接配合 Waline 前端使用
- **Docker 部署** — 多阶段构建，镜像体积小

## 快速开始

### 从源码构建

```bash
# 克隆仓库
git clone <repo-url>
cd waline-server

# 构建
cargo build --release

# 配置数据库
export DATABASE_URL="sqlite://./data/waline.sqlite"

# 设置 JWT 密钥（生产环境必须）
export JWT_TOKEN=your-secret-key

# 启动服务
./target/release/waline-server
```

服务默认监听 `0.0.0.0:8360`。

### Docker 部署

```bash
# 构建镜像
docker build -t waline-rust .

docker run -d \
  -p 8360:8360 \
  -e DATABASE_URL="sqlite:///app/data/waline.sqlite?mode=rwc" \
  -e JWT_TOKEN=your-secret-key \
  -v waline-data:/app/data \
  waline-server
```

> **注意**：使用 SQLite 时，数据库文件所在目录必须对容器内用户（UID 65532）可写。如果挂载宿主机目录，需设置正确权限：
> ```bash
> sudo chown -R 65532:65532 /path/to/data
> sudo chmod -R 755 /path/to/data
> ```

### Docker Compose 示例

```yaml
version: "3"
services:
  waline:
    build: .
    ports:
      - "8360:8360"
    environment:
      - DATABASE_URL=sqlite:///app/data/waline.sqlite?mode=rwc
      - JWT_TOKEN=change-me-to-a-random-string
      - SITE_URL=https://your-site.com
      - SITE_NAME=My Site
    volumes:
      - waline-data:/app/data
    restart: unless-stopped

volumes:
  waline-data:
```

## 数据库配置

数据库 **PostgreSQL、MySQL、SQLite**。只需配置一种数据库即可。

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `DATABASE_URL` | 数据库文件路径 | `sqlite://./data/waline.sqlite` (必填) |

### SQLite 权限配置

使用 SQLite 时，数据库文件所在目录必须对容器内用户可写：

- **Docker volume**：自动处理权限，无需额外配置
- **宿主机目录挂载**：需设置正确权限
  ```bash
  sudo chown -R 65532:65532 /path/to/data
  sudo chmod -R 755 /path/to/data
  ```

数据库文件权限应为 `644`，目录权限应为 `755`。

### 自动建表

首次启动时，服务会根据检测到的数据库类型自动创建所需的表（`{prefix}Comment`、`{prefix}Counter`、`{prefix}Users`），无需手动执行 SQL。

## 完整环境变量参考

### 安全与认证

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `JWT_TOKEN` | JWT 签名密钥，生产环境务必设置 | 数据库密码或空 |
| `SECURE_DOMAINS` | 允许的安全域名，逗号分隔 | — |
| `LOGIN` | 登录模式：`force` 强制登录 | — |
| `COMMENT_AUDIT` | 评论审核模式，新评论需管理员审批 | `false` |
| `FORBIDDEN_WORDS` | 违禁词，逗号分隔 | — |
| `IPQPS` | 单 IP 每秒请求限制 | `60` |
| `AKISMET_KEY` | Akismet 反垃圾 API Key | — |
| `RECAPTCHA_V3_SECRET` | Google reCAPTCHA v3 服务端密钥 | — |
| `RECAPTCHA_V3_KEY` | Google reCAPTCHA v3 前端站点密钥（管理后台登录页使用） | — |
| `TURNSTILE_SECRET` | Cloudflare Turnstile 服务端密钥 | — |
| `TURNSTILE_KEY` | Cloudflare Turnstile 前端站点密钥（管理后台登录页使用） | — |
| `DISABLE_USERAGENT` | 不记录用户 UA | `false` |
| `DISABLE_REGION` | 不记录 IP 归属地 | `false` |

### 站点信息

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `SITE_NAME` | 站点名称 | — |
| `SITE_URL` | 站点 URL | — |
| `SERVER_URL` | 服务端 URL | — |

### SMTP 邮件

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `SMTP_SERVICE` | 邮件服务商预设（见下表） | — |
| `SMTP_HOST` | SMTP 服务器地址 | — |
| `SMTP_PORT` | SMTP 端口 | — |
| `SMTP_SECURE` | 使用 SSL | — |
| `SMTP_USER` | SMTP 用户名 | — |
| `SMTP_PASS` | SMTP 密码 | — |
| `SENDER_EMAIL` | 发件人邮箱 | `SMTP_USER` |
| `SENDER_NAME` | 发件人名称 | — |
| `AUTHOR_EMAIL` | 博主邮箱，匹配此邮箱的评论为博主 | — |
| `DISABLE_AUTHOR_NOTIFY` | 禁止博主通知 | `false` |

**SMTP 服务商预设：**

| 预设值 | 服务商 |
|--------|--------|
| `gmail` | Gmail |
| `outlook` | Outlook |
| `qq` | QQ 邮箱 |
| `163` | 163 邮箱 |
| `126` | 126 邮箱 |
| `aliyun` | 阿里云邮箱 |

设置 `SMTP_SERVICE=gmail` 后无需再设 `SMTP_HOST` 和 `SMTP_PORT`。

### 邮件模板

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `MAIL_SUBJECT` | 回复邮件主题 | — |
| `MAIL_TEMPLATE` | 回复邮件 HTML 模板 | — |
| `MAIL_SUBJECT_ADMIN` | 通知博主邮件主题 | — |
| `MAIL_TEMPLATE_ADMIN` | 通知博主邮件 HTML 模板 | — |

模板变量：`{{site.name}}`、`{{site.url}}`、`{{comment.nick}}`、`{{comment.content}}`、`{{comment.link}}`、`{{comment.mail}}`、`{{comment.ip}}`、`{{comment.url}}`、`{{comment.time}}`

### 通知渠道

| 环境变量 | 说明 |
|---------|------|
| `SC_KEY` | Server酱 SendKey |
| `QYWX_AM` | 企业微信 AgentId|
| `QMSG_KEY` | Qmsg 酱 Key |
| `QQ_ID` | QQ 号（配合 Qmsg 使用） |
| `TG_BOT_TOKEN` | Telegram Bot Token |
| `TG_CHAT_ID` | Telegram Chat ID |
| `PUSH_PLUS_KEY` | PushPlus Key |
| `PUSH_PLUS_TOPIC` | PushPlus 主题 |
| `PUSH_PLUS_CHANNEL` | PushPlus 渠道 |
| `PUSH_PLUS_WEBHOOK` | PushPlus Webhook |
| `DISCORD_WEBHOOK` | Discord Webhook URL |
| `LARK_WEBHOOK` | 飞书/Lark Webhook URL |
| `LARK_SECRET` | 飞书/Lark 签名密钥 |

### 通知模板

| 环境变量 | 说明 |
|---------|------|
| `QQ_TEMPLATE` | QQ 通知模板 |
| `TG_TEMPLATE` | Telegram 通知模板 |
| `WX_TEMPLATE` | 企业微信通知模板 |
| `SC_TEMPLATE` | Server酱通知模板 |
| `DISCORD_TEMPLATE` | Discord 通知模板 |
| `LARK_TEMPLATE` | 飞书通知模板 |

### 头像

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `AVATAR_PROXY` | Gravatar 代理地址，设为 `false` 禁用代理 | — |
| `GRAVATAR_STR` | Gravatar 默认头像策略 | — |

### Markdown 渲染

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `MARKDOWN_CONFIG` | 渲染配置 JSON | `{}` |
| `MARKDOWN_HIGHLIGHT` | 代码高亮，设为 `false` 禁用 | 启用 |
| `MARKDOWN_EMOJI` | Emoji 短代码，设为 `false` 禁用 | 启用 |
| `MARKDOWN_SUB` | 下标 `~text~`，设为 `false` 禁用 | 启用 |
| `MARKDOWN_SUP` | 上标 `^text^`，设为 `false` 禁用 | 启用 |
| `MARKDOWN_TEX` | 数学公式引擎：`mathjax`/`katex`/`false` | `mathjax` |
| `MARKDOWN_MATHJAX` | MathJax 配置 JSON | `{}` |
| `MARKDOWN_KATEX` | KaTeX 配置 JSON | `{}` |

### 其他

| 环境变量 | 说明 | 默认值 |
|---------|------|--------|
| `OAUTH_URL` | OAuth 外部服务地址 | `https://oauth.lithub.cc` |
| `WALINE_ADMIN_MODULE_ASSET_URL` | 管理后台 JS 资源地址 | `//unpkg.com/@waline/admin` |
| `LEVELS` | 用户等级阈值，逗号分隔 | — |
| `LIKE_INC_MAX` | 单次点赞最大增量 | `1` |
| `WEBHOOK` | Webhook URL，评论时 POST 通知 | — |
| `IP2REGION_DB` | IP 归属地数据库路径（ip2region xdb 格式） | — |
| `IP2REGION_DB_V4` | IPv4 归属地数据库路径（优先级高于 IP2REGION_DB） | — |
| `IP2REGION_DB_V6` | IPv6 归属地数据库路径 | — |

### IP 归属地查询

支持通过 [ip2region](https://github.com/lionsoul2014/ip2region) 数据库查询评论者的 IP 归属地。

**数据库下载**：从 [ip2region data 目录](https://github.com/lionsoul2014/ip2region/tree/master/data) 下载 xdb 格式数据库文件。

**配置示例**：
```bash
# IPv4 数据库（推荐）
IP2REGION_DB_V4=/path/to/ip2region.xdb

# 或通用配置（IP2REGION_DB_V4 优先级更高）
IP2REGION_DB=/path/to/ip2region.xdb
```

**禁用归属地记录**：
```bash
DISABLE_REGION=true
```

## API 接口

所有 API 路径以 `/api` 为前缀。

### 评论

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/comment` | 获取评论列表 |
| `POST` | `/api/comment` | 提交新评论 |
| `PUT` | `/api/comment/{id}` | 更新评论 |
| `DELETE` | `/api/comment/{id}` | 删除评论 |

**获取评论参数：**
- `url` — 页面路径
- `type` — 查询类型：`count`(计数) / `recent`(最近) / `list`(列表，管理员)
- `page` / `pageSize` — 分页
- `status` — 筛选状态：`approved` / `waiting` / `spam`

### 文章计数

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/article` | 获取文章访问计数 |
| `POST` | `/api/article` | 更新文章访问计数 |

### 认证

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/token` | 获取当前用户信息 |
| `POST` | `/api/token` | 登录（邮箱 + 密码） |
| `DELETE` | `/api/token` | 登出 |
| `GET` | `/api/token/2fa` | 获取 2FA 设置（QR 码） |
| `POST` | `/api/token/2fa` | 启用 2FA |

### 用户

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/user` | 获取用户列表（管理员） |
| `POST` | `/api/user` | 注册用户（第一个用户自动成为管理员） |
| `PUT` | `/api/user` | 更新用户信息 |
| `DELETE` | `/api/user/{id}` | 删除用户（管理员） |
| `PUT` | `/api/user/password` | 重置密码 |

### OAuth

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/oauth` | OAuth 登录跳转 |

### 数据库管理

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/db` | 导出数据 |
| `POST` | `/api/db` | 导入数据 |
| `PUT` | `/api/db` | 更新记录 |
| `DELETE` | `/api/db` | 清空表 |

### 邮箱验证

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/verification` | 验证邮箱 |

## 管理后台

访问 `/ui/login` 进入管理后台登录页面。后台基于 `@waline/admin` React SPA，与原版 Waline 完全一致。

### 页面路由

| 路径 | 说明 | 权限 |
|------|------|------|
| `/ui/login` | 登录 | 公开 |
| `/ui/register` | 注册 | 公开 |
| `/ui/forgot` | 忘记密码 | 公开 |
| `/ui` | 评论管理 | 管理员 |
| `/ui/user` | 用户管理 | 管理员 |
| `/ui/migration` | 数据迁移 | 管理员 |
| `/ui/profile` | 个人资料 | 登录用户 |

### 自定义管理后台资源

默认从 CDN (`unpkg.com`) 加载 `@waline/admin`，可通过环境变量指定自托管地址：

```bash
export WALINE_ADMIN_MODULE_ASSET_URL=https://your-cdn.com/admin.js
```

## 前端集成

本服务端与原版 Waline 前端完全兼容，在前端初始化时将 `serverURL` 指向本服务即可：

```js
Waline.init({
  el: '#waline',
  serverURL: 'https://your-waline-rust-server.com',
});
```

## 与原版 Waline 的兼容性

| 特性 | 兼容情况 |
|------|---------|
| REST API | 完全兼容 |
| 前端对接 | 完全兼容 |
| 数据库表结构 | 完全兼容，可直接迁移数据 |
| 密码验证 | 兼容 phpass (`$P$`/`$H$`) 和 bcrypt (`$2b$`/`$2a$`/`$2y$`) |
| 环境变量 | 完全兼容，变量名与原版一致 |
| 管理后台 | 完全兼容（`/ui/login`） |
| 数据库适配器 | 仅 PostgreSQL/MySQL/SQLite（原版额外支持 MongoDB、LeanCloud 等） |

### 数据迁移

如果你从原版 Waline 迁移，由于表结构兼容，只需：

1. 将原版数据库导出
2. 导入到新数据库
3. 启动 waline-rust 服务

密码哈希完全兼容，用户无需重置密码。

## 开发

```bash
# 开发构建
cargo build

# 运行检查
cargo check

# 运行测试
cargo test

# 格式化
cargo fmt

# Lint
cargo clippy
```

## 技术栈

| 组件 | 技术 |
|------|------|
| Web 框架 | Axum 0.8 |
| 异步运行时 | Tokio 1 |
| 数据库驱动 | SQLx 0.8 |
| Markdown | comrak (GFM) |
| 代码高亮 | syntect |
| XSS 过滤 | ammonia |
| 邮件 | lettre |
| JWT | jsonwebtoken |
| 密码哈希 | bcrypt + phpass (兼容) |
| 2FA | totp-rs |
| HTTP 客户端 | reqwest |

## License

Apache-2.0

## 仓库镜像

[MyCode](https://git.jetsung.com/jetsung/waline-server) ● [AtomGit](https://atomgit.com/jetsung/waline-server) ● [GitHub](https://github.com/jetsung/waline-server)

