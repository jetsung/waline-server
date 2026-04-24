use comrak::{markdown_to_html, ComrakOptions, ComrakExtensionOptions, ComrakParseOptions, ComrakRenderOptions};
use serde_json::Value;

/// Markdown renderer configuration
pub struct MarkdownConfig {
    pub emoji: bool,
    pub sub: bool,
    pub sup: bool,
    pub tex: Option<String>,
    pub highlight: bool,
    pub extra_config: Value,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            emoji: true,
            sub: true,
            sup: true,
            tex: Some("mathjax".to_string()),
            highlight: true,
            extra_config: Value::Null,
        }
    }
}

/// Render markdown to sanitized HTML.
pub fn render_markdown(content: &str, config: &MarkdownConfig) -> Result<String, String> {
    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: false,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: config.sup,
            footnotes: false,
            description_lists: false,
            front_matter_delimiter: None,
            header_ids: None,
            shortcodes: config.emoji,
            multiline_block_quotes: false,
            math_dollars: config.tex.is_some(),
            math_code: config.tex.is_some(),
            ..Default::default()
        },
        parse: ComrakParseOptions {
            smart: true,
            default_info_string: Some("text".to_string()),
            relaxed_tasklist_matching: true,
            relaxed_autolinks: true,
            ..Default::default()
        },
        render: ComrakRenderOptions {
            hardbreaks: false,
            github_pre_lang: true,
            width: 0,
            unsafe_: false,
            sourcepos: false,
            escaped_char_spans: false,
            ..Default::default()
        },
    };

    let mut html = markdown_to_html(content, &options);

    // Process subscript: ~text~ → <sub>text</sub>
    if config.sub {
        let re = regex::Regex::new(r"~([^~]+)~").unwrap();
        html = re.replace_all(&html, "<sub>$1</sub>").to_string();
    }

    // Apply code syntax highlighting if enabled
    if config.highlight {
        html = highlight_code_blocks(&html);
    }

    // Sanitize HTML to prevent XSS using ammonia
    html = ammonia::clean(&html);

    Ok(html)
}

/// Apply syntax highlighting to fenced code blocks using syntect
fn highlight_code_blocks(html: &str) -> String {
    let re = regex::Regex::new(r#"<pre><code class="language-([^"]*)">([\s\S]*?)</code></pre>"#).unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let lang = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let code = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        // Decode HTML entities in code
        let decoded = code
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        if let Ok(highlighted) = syntect_highlight::highlight_snippet(&decoded, lang) {
            format!(r#"<pre><code class="language-{lang}">{highlighted}</code></pre>"#)
        } else {
            // Fallback: return original
            format!(r#"<pre><code class="language-{lang}">{code}</code></pre>"#)
        }
    }).to_string()
}

/// Parse MARKDOWN_CONFIG JSON string into ComrakOptions-compatible config
pub fn parse_markdown_config(json_str: &str) -> Result<Value, String> {
    if json_str.is_empty() || json_str == "{}" {
        return Ok(Value::Null);
    }
    serde_json::from_str(json_str).map_err(|e| format!("Invalid MARKDOWN_CONFIG: {e}"))
}

/// Helper module for syntect code highlighting
mod syntect_highlight {
    use syntect::parsing::SyntaxSet;
    use syntect::highlighting::ThemeSet;
    use syntect::html::highlighted_html_for_string;

    pub fn highlight_snippet(code: &str, lang: &str) -> Result<String, String> {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = &ts.themes["InspiredGitHub"];

        let syntax = if !lang.is_empty() {
            ss.find_syntax_by_token(lang)
                .or_else(|| ss.find_syntax_by_extension(lang))
                .unwrap_or_else(|| ss.find_syntax_plain_text())
        } else {
            ss.find_syntax_plain_text()
        };

        let html = highlighted_html_for_string(code, &ss, syntax, theme)
            .map_err(|e| format!("Highlighting failed: {e}"))?;
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let config = MarkdownConfig::default();
        let html = render_markdown("**bold** and *italic*", &config).unwrap();
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_xss_sanitization() {
        let config = MarkdownConfig::default();
        let html = render_markdown("<script>alert('xss')</script>", &config).unwrap();
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_gfm_table() {
        let config = MarkdownConfig::default();
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_markdown(md, &config).unwrap();
        assert!(html.contains("<table>"));
    }
}
