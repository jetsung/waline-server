# 通知服务

Waline 支持多种通知渠道，在评论提交后通知博主或回复者。

## 通知渠道概览

| 渠道 | 环境变量 | 说明 |
|------|----------|------|
| 邮件 (SMTP) | SMTP_HOST / SMTP_SERVICE | 邮件通知 |
| Server酱 | SC_KEY | 微信推送 |
| 企业微信 | QYWX_AM | 企业微信应用消息 |
| QQ | QMSG_KEY + QQ_ID | QQ 消息 |
| Telegram | TG_BOT_TOKEN + TG_CHAT_ID | Telegram 消息 |
| PushPlus | PUSH_PLUS_KEY | 多渠道推送 |
| Discord | DISCORD_WEBHOOK | Discord 消息 |
| 飞书/Lark | LARK_WEBHOOK | 飞书机器人 |

## 邮件通知 (SMTP)

### 配置方式一：服务商预设

```bash
SMTP_SERVICE=gmail
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
```

**支持的服务商预设:**

| 预设值 | 服务商 |
|--------|--------|
| gmail | Gmail |
| outlook | Outlook |
| qq | QQ 邮箱 |
| 163 | 163 邮箱 |
| 126 | 126 邮箱 |
| aliyun | 阿里云邮箱 |

### 配置方式二：自定义 SMTP

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
```

### 博主邮箱

```bash
AUTHOR_EMAIL=admin@example.com
```

匹配此邮箱的评论会被标记为博主。

### 禁用博主通知

```bash
DISABLE_AUTHOR_NOTIFY=true
```

## 邮件模板

### 回复通知模板

```bash
MAIL_SUBJECT=[{{site.name}}] 收到新回复
MAIL_TEMPLATE=<h1>{{self.nick}} 回复了您</h1><p>{{self.comment}}</p>
```

### 博主通知模板

```bash
MAIL_SUBJECT_ADMIN=[{{site.name}}] 有新评论
MAIL_TEMPLATE_ADMIN=<h1>{{self.nick}} 评论了</h1><p>{{self.comment}}</p>
```

### 模板变量

| 变量 | 说明 |
|------|------|
| `{{site.name}}` | 站点名称 |
| `{{site.url}}` | 站点 URL |
| `{{site.postUrl}}` | 文章完整 URL |
| `{{self.nick}}` | 评论者昵称 |
| `{{self.mail}}` | 评论者邮箱 |
| `{{self.comment}}` | 评论内容 (HTML) |
| `{{self.url}}` | 文章路径 |
| `{{self.status}}` | 评论状态 |
| `{{parent.nick}}` | 被回复者昵称 |
| `{{parent.comment}}` | 被回复内容 |

## Server酱

```bash
SC_KEY=your_send_key
```

### 自定义模板

```bash
SC_TEMPLATE={{site.name|safe}} 有新评论啦
【评论者昵称】：{{self.nick}}
【评论者邮箱】：{{self.mail}}
【内容】：{{self.comment}}
【地址】：{{site.postUrl}}
```

## 企业微信

```bash
QYWX_AM=corpid,corpsecret,touser,agentid,thumb_media_id
```

参数说明：
- `corpid`: 企业 ID
- `corpsecret`: 应用 Secret
- `touser`: 接收用户
- `agentid`: 应用 ID
- `thumb_media_id`: 封面图片素材 ID

### 代理配置

```bash
QYWX_PROXY=proxy.example.com
QYWX_PROXY_PORT=8080
```

### 自定义模板

```bash
WX_TEMPLATE=💬 {{site.name|safe}}的文章《{{postName}}》有新评论啦
【评论者昵称】：{{self.nick}}
【内容】：{{self.comment}}
```

## QQ (Qmsg)

```bash
QMSG_KEY=your_qmsg_key
QQ_ID=your_qq_number
QMSG_HOST=https://qmsg.zendee.cn  # 可选，自定义 API 地址
```

### 自定义模板

```bash
QQ_TEMPLATE=💬 {{site.name|safe}} 有新评论啦
{{self.nick}} 评论道：
{{self.comment}}
```

## Telegram

```bash
TG_BOT_TOKEN=your_bot_token
TG_CHAT_ID=your_chat_id
```

### 自定义模板

```bash
TG_TEMPLATE=💬 *[{{site.name}}]({{site.url}}) 有新评论啦*

*{{self.nick}}* 回复说：

\`\`\`
{{self.comment}}
\`\`\`
```

## PushPlus

```bash
PUSH_PLUS_KEY=your_key
PUSH_PLUS_TOPIC=topic          # 可选，群推
PUSH_PLUS_CHANNEL=channel      # 可选，渠道
PUSH_PLUS_WEBHOOK=webhook_url  # 可选，回调
```

## Discord

```bash
DISCORD_WEBHOOK=https://discord.com/api/webhooks/xxx/xxx
```

### 自定义模板

```bash
DISCORD_TEMPLATE=💬 {{site.name|safe}} 有新评论啦
【评论者昵称】：{{self.nick}}
【内容】：{{self.comment}}
```

## 飞书/Lark

```bash
LARK_WEBHOOK=https://open.feishu.cn/open-apis/bot/v2/hook/xxx
LARK_SECRET=your_secret  # 可选，签名验证
```

### 自定义模板

```bash
LARK_TEMPLATE=【网站名称】：{{site.name|safe}}
【评论者昵称】：{{self.nick}}
【内容】：{{self.comment}}
```

## 通知流程

```
评论提交
    │
    ▼
┌─────────────────────────────────────┐
│         通知服务 (notify.run)         │
└─────────────────────────────────────┘
    │
    ├─ 检查是否博主评论
    │
    ├─ 检查是否回复博主
    │
    ├─ 检查是否自己回复自己
    │
    ▼
┌─────────────────────────────────────┐
│         博主通知 (非博主评论时)         │
│                                      │
│  优先级:                              │
│  1. Server酱                         │
│  2. 企业微信                          │
│  3. QQ                               │
│  4. Telegram                         │
│  5. PushPlus                         │
│  6. Discord                          │
│  7. 飞书                              │
│  8. 邮件 (以上都未配置时)              │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│         回复通知 (有父评论时)          │
│                                      │
│  条件:                               │
│  - 父评论邮箱有效                     │
│  - 不是自己回复自己                    │
│  - 不是回复博主                       │
│  - 评论状态不是 waiting               │
│                                      │
│  方式: 邮件                           │
└─────────────────────────────────────┘
```

## 通知服务实现

```javascript
// src/service/notify.js

class NotifyService {
  constructor(controller) {
    this.controller = controller;
    // 初始化 SMTP transporter
    if (SMTP_HOST || SMTP_SERVICE) {
      this.transporter = nodemailer.createTransport(config);
    }
  }

  // 邮件通知
  async mail({ to, title, content }, self, parent) {}

  // Server酱
  async wechat({ title, content }, self, parent) {}

  // 企业微信
  async qywxAmWechat({ title, content }, self, parent) {}

  // QQ
  async qq(self, parent) {}

  // Telegram
  async telegram(self, parent) {}

  // PushPlus
  async pushplus({ title, content }, self, parent) {}

  // Discord
  async discord({ title, content }, self, parent) {}

  // 飞书
  async lark({ title, content }, self, parent) {}

  // 主入口
  async run(comment, parent, disableAuthorNotify) {}
}
```

## Webhook

除了内置通知渠道，还支持 Webhook：

```bash
WEBHOOK=https://your-webhook-url
```

评论提交时会 POST 以下数据：

```json
{
  "type": "new_comment",
  "data": {
    "comment": { ... },
    "reply": { ... }
  }
}
```
