# Waline Server 架构概览

## 技术栈

| 组件 | 技术 | 版本 |
|------|------|------|
| Web 框架 | Axum | 0.8 |
| 异步运行时 | Tokio | 1 |
| 数据库驱动 | SQLx | 0.8 |
| Markdown | comrak | GFM |
| 代码高亮 | syntect | - |
| XSS 过滤 | ammonia | - |
| 邮件 | lettre | - |
| JWT | jsonwebtoken | - |
| 密码哈希 | bcrypt + phpass (兼容) | - |
| 2FA | totp-rs | - |
| HTTP 客户端 | reqwest | - |

## 目录结构

```
src/
├── main.rs              # 程序入口
├── app.rs               # Axum 应用构建
├── config.rs            # 配置加载
├── routes.rs            # 路由定义
├── state.rs             # 应用状态
├── error.rs             # 错误处理
├── response.rs          # 响应格式
├── middleware.rs        # 中间件
├── db.rs                # 数据库连接和迁移
│
├── handlers/            # HTTP 处理器
│   ├── comment.rs       # 评论 CRUD
│   ├── user.rs          # 用户管理
│   ├── article.rs       # 文章计数
│   ├── oauth.rs         # OAuth 登录
│   ├── db.rs            # 数据导入导出
│   ├── rss.rs           # RSS 订阅
│   ├── ui.rs            # 管理后台
│   └── verification.rs  # 邮箱验证
│
├── services/            # 业务服务
│   ├── comment.rs       # 评论服务
│   ├── user.rs          # 用户服务
│   ├── article.rs       # 文章服务
│   ├── db.rs            # 数据库服务
│   └── notify/          # 通知服务
│       ├── mod.rs
│       ├── email.rs
│       ├── serverchan.rs
│       ├── wechat.rs
│       ├── qq.rs
│       ├── telegram.rs
│       ├── pushplus.rs
│       ├── discord.rs
│       └── lark.rs
│
├── models/              # 数据模型
│   ├── comment.rs
│   ├── user.rs
│   └── counter.rs
│
├── utils/               # 工具函数
│   ├── jwt.rs           # JWT 处理
│   ├── password.rs      # 密码哈希
│   ├── markdown.rs      # Markdown 渲染
│   ├── avatar.rs        # 头像服务
│   ├── ua.rs            # UA 解析
│   ├── ip.rs            # IP 处理
│   └── spam.rs          # 反垃圾
│
└── locales/             # 国际化
    ├── mod.rs
    └── *.json
```

## 架构分层

```
┌─────────────────────────────────────────────────────────────┐
│                        HTTP Request                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Middleware Pipeline                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │  CORS    │→│  Router  │→│  Auth    │→│  Rate    │       │
│  │          │ │          │ │ (JWT)    │ │  Limit   │       │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       Handlers Layer                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │  Comment   │ │   User     │ │   OAuth    │  ...         │
│  └────────────┘ └────────────┘ └────────────┘              │
│                                                              │
│  处理 HTTP 请求，参数解析，响应格式化                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       Services Layer                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│  │   Notify   │ │  Markdown  │ │   Avatar   │  ...         │
│  └────────────┘ └────────────┘ └────────────┘              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Database Layer                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                    │
│  │PostgreSQL│ │  MySQL   │ │  SQLite  │                    │
│  └──────────┘ └──────────┘ └──────────┘                    │
│                      (SQLx)                                  │
└─────────────────────────────────────────────────────────────┘
```

## 请求处理流程

```
Request → Middleware → Handler → Service → Database → Response
              │
              ├─ JWT Token 解析
              ├─ IP 频率限制
              └─ 用户信息注入 Extension
```

## 数据库表结构

| 表名 | 说明 |
|------|------|
| `wl_Comment` | 评论数据 |
| `wl_Counter` | 文章计数 |
| `wl_Users` | 用户数据 |

详细表结构请参考 [数据库适配器](./database.md)。
