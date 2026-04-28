# 环境变量配置

## 数据库配置

```bash
# 数据库连接字符串（必填）
DATABASE_URL=sqlite://./data/waline.sqlite
# 或 MySQL
DATABASE_URL=mysql://root:password@127.0.0.1:3306/waline
# 或 PostgreSQL
DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/waline
```

## 安全配置

```bash
# JWT 密钥（生产环境必须设置）
JWT_TOKEN=your_secret_key

# 安全域名（逗号分隔）
SECURE_DOMAINS=example.com,www.example.com

# 强制登录
LOGIN=force

# 评论审核
COMMENT_AUDIT=true

# 违禁词（逗号分隔）
FORBIDDEN_WORDS=关键词1,关键词2

# IP 频率限制（秒）
IPQPS=60

# Akismet 反垃圾
AKISMET_KEY=your_akismet_key

# reCAPTCHA v3
RECAPTCHA_V3_SECRET=your_secret
RECAPTCHA_V3_KEY=your_site_key

# Cloudflare Turnstile
TURNSTILE_SECRET=your_secret
TURNSTILE_KEY=your_site_key

# 隐私保护
DISABLE_USERAGENT=true
DISABLE_REGION=true
```

## 站点信息

```bash
SITE_NAME=我的博客
SITE_URL=https://example.com
SERVER_URL=https://waline.example.com
```

## SMTP 邮件配置

### 使用服务商预设

```bash
SMTP_SERVICE=gmail
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
```

支持的服务商：`gmail`, `outlook`, `qq`, `163`, `126`, `aliyun`

### 自定义 SMTP

```bash
SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_SECURE=true
SMTP_USER=your_email@example.com
SMTP_PASS=your_password
```

### 发件人配置

```bash
SENDER_EMAIL=noreply@example.com
SENDER_NAME=Waline 评论通知
AUTHOR_EMAIL=admin@example.com
DISABLE_AUTHOR_NOTIFY=false
```

## 邮件模板

```bash
MAIL_SUBJECT=[{{site.name}}] 收到新回复
MAIL_TEMPLATE=<h1>{{self.nick}} 回复了您</h1><p>{{self.comment}}</p>
MAIL_SUBJECT_ADMIN=[{{site.name}}] 有新评论
MAIL_TEMPLATE_ADMIN=<h1>{{self.nick}} 评论了</h1><p>{{self.comment}}</p>
```

## 通知渠道

### Server酱

```bash
SC_KEY=your_send_key
SC_TEMPLATE=自定义模板
```

### 企业微信

```bash
QYWX_AM=corpid,corpsecret,touser,agentid,thumb_media_id
QYWX_PROXY=proxy.example.com
QYWX_PROXY_PORT=8080
WX_TEMPLATE=自定义模板
```

### QQ

```bash
QMSG_KEY=your_qmsg_key
QQ_ID=your_qq_number
QMSG_HOST=https://qmsg.zendee.cn
QQ_TEMPLATE=自定义模板
```

### Telegram

```bash
TG_BOT_TOKEN=your_bot_token
TG_CHAT_ID=your_chat_id
TG_TEMPLATE=自定义模板
```

### PushPlus

```bash
PUSH_PLUS_KEY=your_key
PUSH_PLUS_TOPIC=topic
PUSH_PLUS_CHANNEL=channel
PUSH_PLUS_WEBHOOK=webhook_url
```

### Discord

```bash
DISCORD_WEBHOOK=https://discord.com/api/webhooks/xxx/xxx
DISCORD_TEMPLATE=自定义模板
```

### 飞书

```bash
LARK_WEBHOOK=https://open.feishu.cn/open-apis/bot/v2/hook/xxx
LARK_SECRET=your_secret
LARK_TEMPLATE=自定义模板
```

## 头像配置

```bash
# Gravatar 代理
AVATAR_PROXY=https://cdn.libravatar.org/avatar/

# Gravatar 默认头像策略
GRAVATAR_STR=自定义模板
```

## Markdown 配置

```bash
# 基础配置 (JSON)
MARKDOWN_CONFIG={"breaks":true,"linkify":true}

# 代码高亮
MARKDOWN_HIGHLIGHT=false

# Emoji
MARKDOWN_EMOJI=false

# 上标/下标
MARKDOWN_SUB=false
MARKDOWN_SUP=false

# 数学公式
MARKDOWN_TEX=mathjax  # mathjax / katex / false
MARKDOWN_MATHJAX={"inlineMath":[["$","$"]]}
MARKDOWN_KATEX={"throwOnError":false}
```

## 其他配置

```bash
# OAuth 服务地址
OAUTH_URL=https://oauth.lithub.cc

# 管理后台资源地址
WALINE_ADMIN_MODULE_ASSET_URL=https://unpkg.com/@waline/admin

# 用户等级阈值
LEVELS=0,10,50,100,500

# 点赞最大增量
LIKE_INC_MAX=1

# Webhook
WEBHOOK=https://your-webhook-url

# IP 归属地数据库（ip2region xdb 格式）
# 数据库下载地址：https://github.com/lionsoul2014/ip2region/tree/master/data
IP2REGION_DB=/path/to/ip2region.xdb
IP2REGION_DB_V4=/path/to/ip2region.xdb
IP2REGION_DB_V6=/path/to/ip2region.xdb
```

## IP 归属地查询

Waline 支持通过 ip2region 数据库查询评论者的 IP 归属地。

### 数据库下载

从 [ip2region data 目录](https://github.com/lionsoul2014/ip2region/tree/master/data) 下载 xdb 格式的数据库文件：

- `ip2region.xdb` - IPv4 归属地数据库
- `ip2region.xdb` - IPv6 归属地数据库（同名但内容不同）

### 配置方式

```bash
# IPv4 数据库（推荐）
IP2REGION_DB_V4=/path/to/ip2region.xdb

# 或使用通用配置（IP2REGION_DB_V4 优先级更高）
IP2REGION_DB=/path/to/ip2region.xdb

# IPv6 数据库（可选）
IP2REGION_DB_V6=/path/to/ip2region.xdb
```

### 禁用 IP 归属地记录

```bash
# 禁止记录 IP 归属地
DISABLE_REGION=true
```

## 完整配置示例

```bash
# 数据库
DATABASE_URL=sqlite://./data/waline.sqlite

# 安全
JWT_TOKEN=your-very-long-secret-key
SECURE_DOMAINS=example.com
COMMENT_AUDIT=true

# 站点
SITE_NAME=我的博客
SITE_URL=https://example.com

# 邮件
SMTP_SERVICE=gmail
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
AUTHOR_EMAIL=admin@example.com

# 通知
TG_BOT_TOKEN=your_bot_token
TG_CHAT_ID=your_chat_id

# 头像
AVATAR_PROXY=https://cdn.libravatar.org/avatar/
```
