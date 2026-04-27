# Waline Server API 接口文档

所有 API 路径以 `/api` 为前缀。

## 评论接口 `/api/comment`

### GET /api/comment - 获取评论列表

获取指定路径的评论列表，支持分页和排序。

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| path | String | 是 | 页面路径 |
| page | Number | 否 | 页码，默认 1 |
| pageSize | Number | 否 | 每页数量，默认 10，最大 100 |
| sortBy | String | 否 | 排序: insertedAt_desc / insertedAt_asc / like_desc |

**响应:**

```json
{
  "page": 1,
  "totalPages": 5,
  "pageSize": 10,
  "count": 50,
  "data": [
    {
      "objectId": "abc123",
      "nick": "用户名",
      "mail": "md5哈希",
      "link": "https://...",
      "avatar": "https://...",
      "browser": "Chrome 120",
      "os": "Windows 11",
      "comment": "<p>评论内容</p>",
      "url": "/post/1",
      "pid": "",
      "rid": "",
      "like": 0,
      "time": 1700000000000,
      "children": []
    }
  ]
}
```

### GET /api/comment?type=list - 管理员获取评论列表

管理员后台获取评论列表，支持筛选。

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | Number | 否 | 页码 |
| pageSize | Number | 否 | 每页数量 |
| owner | String | 否 | mine: 我的评论 |
| status | String | 否 | approved/waiting/spam |
| keyword | String | 否 | 关键词搜索 |

**响应:**

```json
{
  "page": 1,
  "totalPages": 5,
  "pageSize": 10,
  "spamCount": 10,
  "waitingCount": 5,
  "data": [...]
}
```

### GET /api/comment?type=count - 获取评论数量

获取一个或多个路径的评论数量。

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| url | String/Array | 是 | 页面路径，多个用逗号分隔 |

**响应:**

```
// 单个路径
300

// 多个路径
[300, 100, 50]
```

### GET /api/comment?type=recent - 获取最近评论

获取最近的评论列表。

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| count | Number | 否 | 数量，默认 10，最大 50 |

**响应:**

```json
[
  {
    "objectId": "abc123",
    "nick": "用户名",
    "comment": "评论内容",
    "url": "/post/1",
    ...
  }
]
```

### POST /api/comment - 提交评论

提交新评论。

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| nick | String | 是* | 昵称（未登录时必填） |
| mail | String | 是* | 邮箱（未登录时必填） |
| link | String | 否 | 个人链接 |
| comment | String | 是 | 评论内容 |
| url | String | 是 | 文章路径 |
| ua | String | 否 | User Agent |
| pid | String | 否 | 父评论 ID |
| rid | String | 否 | 根评论 ID |
| at | String | 否 | @用户名 |

**处理流程:**

1. IP 黑名单检查
2. 重复内容检测
3. IP 频率限制（IPQPS 环境变量，默认 60 秒）
4. Akismet 反垃圾检测
5. 违禁词过滤
6. preSave 钩子
7. 保存到数据库
8. 发送通知
9. postSave 钩子

### PUT /api/comment/:id - 更新评论

更新评论内容或状态。

**权限:**
- 管理员: 可更新所有字段
- 评论作者: 只能更新自己的评论内容
- 未登录用户: 可点赞

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| comment | String | 评论内容 |
| status | String | 状态 (管理员) |
| sticky | Boolean | 置顶 (管理员) |
| like | Boolean | 点赞 |

### DELETE /api/comment/:id - 删除评论

删除评论及其所有回复。

**权限:** 管理员或评论作者

---

## 用户接口 `/api/user`

### GET /api/user - 获取用户列表

**未登录/普通用户:** 返回评论数最多的用户列表

**管理员:** 返回分页用户列表

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| page | Number | 页码 (管理员) |
| pageSize | Number | 每页数量 |
| email | String | 按邮箱查询 (管理员) |

### POST /api/user - 注册用户

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| display_name | String | 是 | 昵称 |
| email | String | 是 | 邮箱 |
| password | String | 是 | 密码 |
| url | String | 否 | 个人链接 |

**处理逻辑:**

1. 检查邮箱是否已注册
2. 第一个用户自动成为管理员
3. 如果配置了邮件服务，发送验证邮件
4. 否则直接创建为 guest 类型

### PUT /api/user - 更新用户信息

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| display_name | String | 昵称 |
| email | String | 邮箱 |
| url | String | 个人链接 |
| avatar | String | 头像 URL |
| password | String | 新密码 |
| label | String | 用户标签 |
| 2fa | String | 2FA 密钥 |
| github/wechat/... | String | OAuth 绑定 |

### PUT /api/user/:id - 更新指定用户 (管理员)

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| type | String | 用户类型: administrator/guest/banned |

### DELETE /api/user/:id - 删除/禁用用户

- 验证中用户: 直接删除
- 正常用户: 设置为 banned 状态

---

## 认证接口 `/api/token`

### GET /api/token - 获取当前用户信息

返回 JWT Token 解析后的用户信息。

### POST /api/token - 登录

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| email | String | 是 | 邮箱 |
| password | String | 是 | 密码 |
| code | String | 条件 | 2FA 验证码 |

**响应:**

```json
{
  "objectId": "user123",
  "display_name": "用户名",
  "email": "user@example.com",
  "type": "administrator",
  "avatar": "https://...",
  "token": "jwt_token"
}
```

### DELETE /api/token - 登出

客户端清除 Token 即可。

---

## 双因素认证 `/api/token/2fa`

### GET /api/token/2fa - 获取 2FA 设置

**已登录:** 返回当前用户的 2FA 密钥和 QR 码 URL

**未登录 + email 参数:** 返回该用户是否启用了 2FA

**响应:**

```json
{
  "otpauth_url": "otpauth://totp/waline_xxx?secret=xxx",
  "secret": "BASE32SECRET"
}
```

### POST /api/token/2fa - 启用 2FA

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| secret | String | 是 | 2FA 密钥 |
| code | String | 是 | 6 位验证码 |

---

## 密码重置 `/api/user/password`

### PUT /api/user/password - 发送重置密码邮件

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| email | String | 是 | 邮箱 |

**前提:** 必须配置 SMTP 服务

---

## 文章计数 `/api/article`

### GET /api/article - 获取文章计数

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| path | Array | 是 | 页面路径数组 |
| type | Array | 否 | 计数类型数组，默认 ["time"] |

**响应:**

```json
[
  { "time": 100, "like": 5 },
  { "time": 50, "like": 2 }
]
```

### POST /api/article - 更新文章计数

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| path | String | 是 | 页面路径 |
| type | String | 否 | 计数类型，默认 time |
| action | String | 否 | inc(增加) / desc(减少) |

---

## OAuth 登录 `/api/oauth`

### GET /api/oauth - OAuth 登录跳转

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | String | 是 | OAuth 类型: github/wechat/google 等 |
| redirect | String | 否 | 登录后跳转地址 |

**流程:**

重定向到 OAuth 提供商授权页面。

### GET /api/oauth/callback - OAuth 回调处理

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | String | 是 | OAuth 类型 |
| code | String | 是 | OAuth 回调码 |
| state | String | 否 | OAuth 状态 |

**流程:**

1. 获取 OAuth 用户信息
2. 检查是否已绑定账户
3. 已绑定: 签发 Token，重定向到目标页面
4. 未绑定 + 已登录: 绑定到当前账户
5. 未绑定 + 未登录: 创建新账户，签发 Token

---

## 邮箱验证 `/api/verification`

### GET /api/verification - 验证邮箱

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| token | String | 是 | 验证码 |
| email | String | 是 | 邮箱 |

验证成功后重定向到登录页。

---

## 数据库管理 `/api/db`

**权限:** 仅管理员

### GET /api/db - 导出数据

导出所有表数据。

**响应:**

```json
{
  "type": "waline",
  "version": 1,
  "time": 1700000000000,
  "tables": ["Comment", "Counter", "Users"],
  "data": {
    "Comment": [...],
    "Counter": [...],
    "Users": [...]
  }
}
```

### POST /api/db - 导入数据

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| table | String | 是 | 表名 (Query 参数) |
| * | Object | 是 | 记录数据 (Body) |

### PUT /api/db - 更新记录

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| table | String | 是 | 表名 |
| objectId | String | 是 | 记录 ID |

### DELETE /api/db - 清空表

**参数:**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| table | String | 是 | 表名 |

---

## RSS 订阅 `/api/comment/rss`

### GET /api/comment/rss - 获取 RSS

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| path | String | 指定路径的评论 |
| email | String | 回复给该邮箱的评论 |
| user_id | String | 回复给该用户的评论 |
| count | Number | 数量，默认 20，最大 50 |

**响应:** RSS 2.0 XML 格式

---

## 首页示例 `/`

### GET / - 示例页面

返回一个包含 Waline 客户端的 HTML 页面，用于测试。
