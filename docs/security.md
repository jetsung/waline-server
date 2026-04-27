# 安全机制

Waline 提供多层安全防护机制。

## 认证与授权

### JWT Token 认证

```bash
JWT_TOKEN=your_secret_key
```

JWT Token 用于用户身份验证，生产环境必须设置。

**Token 生成:**

```javascript
const token = jwt.sign(userId, jwtKey);
```

**Token 验证:**

```javascript
// logic/base.js __before()
const userId = jwt.verify(token, jwtKey);
```

### 密码哈希

使用 phpass 兼容的密码哈希算法：

```javascript
// 加密
const hash = pwdHash.hashPassword(password);

// 验证
const valid = pwdHash.checkPassword(password, storedHash);
```

支持以下哈希格式：
- phpass: `$P$` / `$H$` 前缀
- bcrypt: `$2b$` / `$2a$` / `$2y$` 前缀

### 双因素认证 (2FA/TOTP)

使用 speakeasy 实现 TOTP 双因素认证。

**启用流程:**

1. GET `/api/token/2fa` 获取密钥和 QR 码
2. 用户扫描 QR 码添加到验证器应用
3. POST `/api/token/2fa` 提交验证码确认启用

**登录验证:**

```javascript
const verified = speakeasy.totp.verify({
  secret: twoFactorAuthSecret,
  encoding: 'base32',
  token: code,
  window: 2,
});
```

### OAuth 登录

支持多种 OAuth 提供商：

```bash
OAUTH_URL=https://oauth.lithub.cc
```

支持的 OAuth 类型：
- GitHub
- Google
- WeChat
- QQ
- Weibo
- Facebook
- Twitter
- 等等...

**OAuth 流程:**

```
1. GET /api/oauth?type=github
   → 重定向到 GitHub 授权页

2. 用户授权后回调
   GET /api/oauth?type=github&code=xxx
   → 获取用户信息
   → 创建/更新用户
   → 返回 JWT Token
```

## 安全域名

```bash
SECURE_DOMAINS=example.com,www.example.com
```

限制只有指定域名可以访问 API。

**检查逻辑:**

```javascript
// logic/base.js referrerCheck()
const referrer = ctx.referrer(true);
const isSafe = secureDomains.some(domain => 
  domain === referrer || domain.test(referrer)
);
```

支持正则表达式：

```bash
SECURE_DOMAINS=/.*\.example\.com$/
```

## 登录模式

```bash
LOGIN=force
```

设置为 `force` 时，未登录用户无法提交评论。

## 评论审核

```bash
COMMENT_AUDIT=true
```

启用后，新评论默认为 `waiting` 状态，需要管理员审核。

## 反垃圾机制

### Akismet

```bash
AKISMET_KEY=your_akismet_key
```

使用 Akismet 服务检测垃圾评论。

**检测流程:**

```javascript
const spam = await akismet.checkComment({
  user_ip: comment.ip,
  permalink: SITE_URL + comment.url,
  comment_author: comment.nick,
  comment_content: comment.comment,
});
```

### 违禁词过滤

```bash
FORBIDDEN_WORDS=关键词1,关键词2,关键词3
```

评论内容包含违禁词时，自动标记为垃圾。

```javascript
const regexp = new RegExp('(' + forbiddenWords.join('|') + ')', 'ig');
if (regexp.test(comment)) {
  data.status = 'spam';
}
```

### IP 频率限制

```bash
IPQPS=60
```

限制同一 IP 在指定秒数内只能提交一条评论。

```javascript
const recent = await model.select({
  ip: ctx.ip,
  insertedAt: ['>', new Date(Date.now() - IPQPS * 1000)]
});
if (!think.isEmpty(recent)) {
  return fail('Comment too fast!');
}
```

### IP 黑名单

```javascript
// config.js
disallowIPList: []
```

可通过配置设置 IP 黑名单。

### reCAPTCHA v3

```bash
RECAPTCHA_V3_SECRET=your_secret
RECAPTCHA_V3_KEY=your_site_key
```

使用 Google reCAPTCHA v3 验证。

**验证流程:**

```javascript
const response = await fetch('https://recaptcha.net/recaptcha/api/siteverify', {
  method: 'POST',
  body: `secret=${secret}&response=${token}&remoteip=${ip}`
});
if (!response.success) {
  ctx.throw(403);
}
```

### Cloudflare Turnstile

```bash
TURNSTILE_SECRET=your_secret
TURNSTILE_KEY=your_site_key
```

使用 Cloudflare Turnstile 验证。

**验证流程:**

```javascript
const response = await fetch('https://challenges.cloudflare.com/turnstile/v0/siteverify', {
  method: 'POST',
  body: `secret=${secret}&response=${token}&remoteip=${ip}`
});
if (!response.success) {
  ctx.throw(403);
}
```

## XSS 防护

使用 DOMPurify 过滤 HTML：

```javascript
// service/markdown/xss.js
const sanitize = (content) =>
  DOMPurify.sanitize(content, {
    FORBID_TAGS: ['form', 'input', 'style'],
    FORBID_ATTR: ['autoplay', 'style'],
  });
```

**安全措施:**

1. 禁止危险标签: `<form>`, `<input>`, `<style>`
2. 禁止危险属性: `autoplay`, `style`
3. 所有链接强制 `target="_blank"`
4. 所有链接添加 `rel="ugc nofollow noreferrer noopener"`

## 隐私保护

### 禁用 User Agent 记录

```bash
DISABLE_USERAGENT=true
```

### 禁用 IP 归属地记录

```bash
DISABLE_REGION=true
```

### 管理员权限

管理员可以看到：
- 评论者邮箱
- 评论者 IP
- 完整 IP 归属地

普通用户无法看到这些信息。

## 用户类型

| 类型 | 说明 |
|------|------|
| administrator | 管理员，拥有所有权限 |
| guest | 普通用户 |
| verify:token:expire | 待验证用户 |
| banned | 被禁用的用户 |

## 权限控制

### 评论权限

| 操作 | 未登录 | 登录用户 | 管理员 |
|------|--------|----------|--------|
| 查看已审核评论 | ✓ | ✓ | ✓ |
| 查看待审核评论 | ✗ | 自己的 | ✓ |
| 提交评论 | ✓* | ✓ | ✓ |
| 编辑评论 | ✗ | 自己的 | ✓ |
| 删除评论 | ✗ | 自己的 | ✓ |

*取决于 LOGIN 配置

### 管理功能权限

所有管理功能（用户管理、数据导入导出等）仅管理员可访问。

## 安全流程图

```
请求到达
    │
    ▼
┌─────────────────┐
│  安全域名检查    │
└─────────────────┘
    │ 通过
    ▼
┌─────────────────┐
│  JWT Token 解析  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  用户信息注入    │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  权限检查        │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  验证码检查      │ (登录/注册时)
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  业务逻辑处理    │
└─────────────────┘
```

## 评论提交流程安全检查

```
POST /api/comment
    │
    ├─ 1. IP 黑名单检查
    │      └─ 在黑名单 → 403
    │
    ├─ 2. 登录检查 (LOGIN=force)
    │      └─ 未登录 → 401
    │
    ├─ 3. 验证码检查 (reCAPTCHA/Turnstile)
    │      └─ 验证失败 → 403
    │
    ├─ 4. 重复内容检测
    │      └─ 重复 → 拒绝
    │
    ├─ 5. IP 频率限制
    │      └─ 频繁 → 拒绝
    │
    ├─ 6. Akismet 检测
    │      └─ 垃圾 → status=spam
    │
    ├─ 7. 违禁词过滤
    │      └─ 包含 → status=spam
    │
    ├─ 8. 审核模式检查
    │      └─ COMMENT_AUDIT=true → status=waiting
    │
    └─ 9. 保存评论
```
