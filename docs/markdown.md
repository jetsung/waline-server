# Markdown 渲染

Waline 使用 markdown-it 作为 Markdown 解析器，支持丰富的扩展语法。

## 基础配置

```bash
MARKDOWN_CONFIG={"breaks":true,"linkify":true}
```

默认配置：
- `breaks: true` - 换行符转换为 `<br>`
- `linkify: true` - 自动识别 URL
- `typographer: true` - 启用排版优化
- `html: true` - 允许 HTML（用于 Emoji）

## 代码高亮

```bash
MARKDOWN_HIGHLIGHT=false  # 禁用代码高亮
```

使用 Prism.js 进行代码高亮，支持所有主流语言。

**高亮流程:**

```javascript
highlight: (code, lang) => {
  const highlighter = resolveHighlighter(lang);
  return highlighter ? highlighter(code) : '';
}
```

## Emoji 支持

```bash
MARKDOWN_EMOJI=false  # 禁用 Emoji
```

支持 Emoji 短代码：

```
:smile: → 😄
:heart: → ❤️
```

使用 `markdown-it-emoji` 插件实现。

## 上标和下标

```bash
MARKDOWN_SUB=false  # 禁用下标
MARKDOWN_SUP=false  # 禁用上标
```

**语法:**

```
H~2~O → H₂O
E=mc^2^ → E=mc²
```

## 数学公式

### MathJax (默认)

```bash
MARKDOWN_TEX=mathjax
MARKDOWN_MATHJAX={"inlineMath":[["$","$"]]}
```

### KaTeX

```bash
MARKDOWN_TEX=katex
MARKDOWN_KATEX={"throwOnError":false}
```

### 禁用数学公式

```bash
MARKDOWN_TEX=false
```

**语法:**

```
行内公式: $E=mc^2$
块级公式:
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

## XSS 过滤

所有 Markdown 渲染结果都会经过 DOMPurify 过滤。

**过滤规则:**

```javascript
DOMPurify.sanitize(content, {
  FORBID_TAGS: ['form', 'input', 'style'],
  FORBID_ATTR: ['autoplay', 'style'],
});
```

**链接安全处理:**

```javascript
DOMPurify.addHook('afterSanitizeAttributes', (node) => {
  if ('target' in node && node.href) {
    node.setAttribute('target', '_blank');
    node.setAttribute('rel', 'ugc nofollow noreferrer noopener');
  }
});
```

## 渲染流程

```
原始评论内容
    │
    ▼
┌─────────────────────┐
│   markdown-it 解析   │
│                      │
│  - GFM 语法          │
│  - Emoji 短代码      │
│  - 上标/下标         │
│  - 数学公式          │
│  - 代码高亮          │
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│   DOMPurify 过滤     │
│                      │
│  - 移除危险标签      │
│  - 移除危险属性      │
│  - 链接安全处理      │
└─────────────────────┘
    │
    ▼
  安全的 HTML
```

## 支持的 Markdown 语法

### GFM (GitHub Flavored Markdown)

- 标题
- 段落
- 强调 (粗体、斜体、删除线)
- 列表 (有序、无序)
- 任务列表
- 代码块 (围栏式、缩进式)
- 表格
- 链接
- 图片
- 引用
- 水平分割线

### 扩展语法

| 语法 | 说明 |
|------|------|
| `:emoji:` | Emoji 短代码 |
| `~text~` | 下标 |
| `^text^` | 上标 |
| `$...$` | 行内数学公式 |
| `$$...$$` | 块级数学公式 |

## 渲染器实现

```javascript
// src/service/markdown/index.js

const getMarkdownParser = () => {
  const { markdown = {} } = think.config();
  const { config = {}, plugin = {} } = markdown;

  // 创建 markdown-it 实例
  const markdownIt = MarkdownIt({
    breaks: true,
    linkify: true,
    typographer: true,
    html: true,
    highlight: (code, lang) => {
      const highlighter = resolveHighlighter(lang);
      return highlighter ? highlighter(code) : '';
    },
    ...config,
  });

  // Emoji 插件
  if (plugin.emoji !== false) {
    markdownIt.use(emojiPlugin.full, plugin.emoji || {});
  }

  // 下标插件
  if (plugin.sub !== false) {
    markdownIt.use(subPlugin);
  }

  // 上标插件
  if (plugin.sup !== false) {
    markdownIt.use(supPlugin);
  }

  // 数学公式插件
  if (plugin.tex === 'katex') {
    markdownIt.use(katexPlugin, { ...plugin.katex, output: 'mathml' });
  } else if (plugin.tex !== false) {
    markdownIt.use(mathjaxPlugin, plugin.mathjax);
  }

  // 返回渲染函数
  return (content) => sanitize(markdownIt.render(content));
};
```

## 代码高亮实现

```javascript
// src/service/markdown/highlight.js

const Prism = require('prismjs');

const resolveHighlighter = (lang) => {
  const language = Prism.languages[lang];
  if (!language) return null;
  
  return (code) => {
    const highlighted = Prism.highlight(code, language, lang);
    return `<pre class="language-${lang}"><code class="language-${lang}">${highlighted}</code></pre>`;
  };
};
```

## 自定义配置示例

### 禁用所有扩展

```bash
MARKDOWN_HIGHLIGHT=false
MARKDOWN_EMOJI=false
MARKDOWN_SUB=false
MARKDOWN_SUP=false
MARKDOWN_TEX=false
```

### 自定义 MathJax

```bash
MARKDOWN_MATHJAX={"inlineMath":[["\\(","\\)"]],"displayMath":[["\\[","\\]"]]}
```

### 自定义 KaTeX

```bash
MARKDOWN_TEX=katex
MARKDOWN_KATEX={"throwOnError":false,"strict":"ignore"}
```
