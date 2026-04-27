# 插件开发

Waline 支持通过插件系统扩展功能。

## 插件系统架构

### 中间件插件

```javascript
// 插件可以注册中间件
const middlewares = think.getPluginMiddlewares();
// 返回中间件数组，会被 compose 处理
```

### 钩子插件

```javascript
// 插件可以注册钩子函数
const hooks = think.getPluginHook('preSave');
// 返回钩子函数数组
```

## 可用钩子

| 钩子名称 | 触发时机 | 参数 | 返回值 |
|----------|----------|------|--------|
| preSave | 评论保存前 | comment data | 错误信息则中断 |
| postSave | 评论保存后 | saved comment, parent comment | - |
| preUpdate | 评论更新前 | update data | 错误信息则中断 |
| postUpdate | 评论更新后 | update data | - |
| preDelete | 评论删除前 | comment id | 错误信息则中断 |
| postDelete | 评论删除后 | comment id | - |

## 插件配置

插件通过 `plugins` 配置项注册：

```javascript
// 配置
think.config('plugins', [
  {
    // 中间件
    middlewares: [
      async (ctx, next) => {
        // 前置处理
        await next();
        // 后置处理
      }
    ],
    
    // 钩子
    hooks: {
      preSave: async (data) => {
        // 处理评论数据
        // 返回错误信息则中断保存
      },
      postSave: async (comment, parent) => {
        // 评论保存后的处理
      }
    }
  }
]);
```

## 插件开发示例

### 示例 1: 敏感词过滤插件

```javascript
const sensitiveWords = ['badword1', 'badword2'];

const plugin = {
  hooks: {
    preSave: (data) => {
      const content = data.comment;
      for (const word of sensitiveWords) {
        if (content.includes(word)) {
          return { errmsg: '评论包含敏感词' };
        }
      }
    }
  }
};

// 注册插件
think.config('plugins', [plugin]);
```

### 示例 2: 评论统计插件

```javascript
const plugin = {
  hooks: {
    postSave: async (comment, parent) => {
      // 发送到统计服务
      await fetch('https://analytics.example.com/track', {
        method: 'POST',
        body: JSON.stringify({
          event: 'new_comment',
          data: comment
        })
      });
    }
  }
};
```

### 示例 3: 请求日志中间件

```javascript
const plugin = {
  middlewares: [
    async (ctx, next) => {
      const start = Date.now();
      await next();
      const duration = Date.now() - start;
      console.log(`${ctx.method} ${ctx.url} - ${duration}ms`);
    }
  ]
};
```

### 示例 4: IP 黑名单中间件

```javascript
const blacklist = ['192.168.1.100', '10.0.0.50'];

const plugin = {
  middlewares: [
    async (ctx, next) => {
      if (blacklist.includes(ctx.ip)) {
        ctx.status = 403;
        ctx.body = { error: 'IP 被禁止访问' };
        return;
      }
      await next();
    }
  ]
};
```

## 钩子调用流程

```javascript
// controller/rest.js

async hook(name, ...args) {
  // 1. 获取配置中的钩子函数
  const fn = this.config(name);
  
  // 2. 获取插件中的钩子函数
  const plugins = think.getPluginHook(name);
  
  // 3. 合并
  if (think.isFunction(fn)) {
    plugins.unshift(fn);
  }
  
  // 4. 依次执行
  for (const plugin of plugins) {
    if (!think.isFunction(plugin)) continue;
    
    const resp = await plugin.call(this, ...args);
    
    // 5. 任一函数返回值则中断
    if (resp) {
      return resp;
    }
  }
}
```

## 自定义存储适配器

```javascript
// 通过 customModel 配置
think.config('customModel', (modelName, ctx) => {
  if (modelName === 'Comment') {
    return new CustomCommentStorage();
  }
  // 返回 null 使用默认适配器
  return null;
});

class CustomCommentStorage {
  constructor() {
    this.tableName = 'Comment';
  }
  
  async select(where, options) {
    // 自定义查询逻辑
  }
  
  async count(where, options) {
    // 自定义计数逻辑
  }
  
  async add(data, options) {
    // 自定义添加逻辑
  }
  
  async update(data, where) {
    // 自定义更新逻辑
  }
  
  async delete(where) {
    // 自定义删除逻辑
  }
}
```

## 自定义头像服务

```javascript
// 通过 avatarUrl 配置
think.config('avatarUrl', async (comment) => {
  // 自定义头像 URL 生成逻辑
  if (comment.mail.endsWith('@company.com')) {
    return `https://internal-cdn.company.com/avatar/${comment.mail}`;
  }
  // 返回空使用默认逻辑
  return null;
});
```

## 自定义密码哈希

```javascript
// 通过 encryptPassword 配置
think.config('encryptPassword', CustomPasswordHash);

class CustomPasswordHash {
  hashPassword(password) {
    // 自定义加密逻辑
    return customHash(password);
  }
  
  checkPassword(password, storedHash) {
    // 自定义验证逻辑
    return customVerify(password, storedHash);
  }
}
```

## 自定义国际化

```javascript
// 通过 locales 配置
think.config('locales', {
  'zh-cn': {
    'USER_EXIST': '用户已存在',
    'Duplicate Content': '重复内容',
    // ...
  },
  'en-us': {
    'USER_EXIST': 'User already exists',
    'Duplicate Content': 'Duplicate content',
    // ...
  }
});
```

## 自定义 XSS 过滤

```javascript
// 通过 domPurify 配置
think.config('domPurify', {
  FORBID_TAGS: ['form', 'input', 'style', 'script'],
  FORBID_ATTR: ['autoplay', 'style', 'onerror'],
  ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'a', 'img']
});
```

## 插件最佳实践

1. **错误处理**: 钩子函数应该捕获异常，避免影响主流程
2. **异步操作**: 钩子函数支持 async/await
3. **返回值**: preSave/preUpdate/preDelete 返回错误信息会中断操作
4. **性能**: 避免在钩子中执行耗时操作
5. **幂等性**: postSave 等钩子可能被重试，需要保证幂等

## 插件加载顺序

```
1. 配置中的钩子函数
2. 插件数组中的钩子函数（按数组顺序）
3. 任一函数返回值则中断后续执行
```
