# Waline Server 文档

Waline Rust 是 Waline 评论系统的 Rust 重写版本，使用 Axum + SQLx 构建，提供与原版 Node.js 服务端完全兼容的 REST API。

## 文档目录

| 文档 | 说明 |
|------|------|
| [架构概览](./architecture.md) | 项目整体架构、技术栈和目录结构 |
| [API 接口](./api.md) | REST API 接口文档 |
| [数据库适配器](./database.md) | 支持的数据库类型和配置 |
| [通知服务](./notification.md) | 邮件和第三方通知渠道 |
| [安全机制](./security.md) | 认证、授权、反垃圾等安全功能 |
| [Markdown 渲染](./markdown.md) | Markdown 解析和渲染功能 |
| [流程图](./flowcharts.md) | 核心业务流程图 |
| [环境变量配置](./configuration.md) | 完整环境变量参考 |

## 功能特性

- **3 种数据库适配器** — PostgreSQL、MySQL、SQLite（自动检测）
- **8 种通知渠道** — Server酱、企业微信、QQ、Telegram、PushPlus、Discord、飞书/Lark、邮件(SMTP)
- **安全认证** — JWT 令牌、phpass/bcrypt 密码兼容、双因素认证(2FA/TOTP)、OAuth2 登录
- **反垃圾** — Akismet、违禁词过滤、IP 频率限制、reCAPTCHA v3、Cloudflare Turnstile
- **Markdown 渲染** — GFM 语法、代码高亮(syntect)、Emoji 短代码、LaTeX 数学公式、XSS 过滤
- **完整 API** — 与原版 Waline 完全兼容的 REST API，可直接配合 Waline 前端使用
- **Docker 部署** — 多阶段构建，镜像体积小

## 快速链接

- [项目源码](../)
- [环境变量配置](./configuration.md)
- [与原版 Waline 的兼容性](./database.md#与原版-waline-的差异)
