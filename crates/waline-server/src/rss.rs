use waline_common::models::Comment;
use waline_core::config::Config;

/// Generate RSS 2.0 XML feed from a list of comments
pub fn generate_rss_feed(
    comments: &[Comment],
    site_name: &str,
    site_url: &str,
    config: &Config,
) -> String {
    let server_url = config.server_url.as_deref().unwrap_or(site_url);
    let now = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT");

    let mut items = String::new();
    for comment in comments {
        let link = format!("{server_url}{}", comment.url.as_deref().unwrap_or(""));
        let author = comment.nick.as_deref().unwrap_or("Anonymous");
        let pub_date = comment.inserted_at.format("%a, %d %b %Y %H:%M:%S GMT");
        items.push_str(&format!(
            r#"    <item>
      <title>Comment by {author}</title>
      <link>{link}</link>
      <description>{}</description>
      <author>{author}</author>
      <pubDate>{pub_date}</pubDate>
    </item>
"#,
            xml_escape(&comment.comment)
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>{site_name} - Comments</title>
    <link>{site_url}</link>
    <description>Comment feed for {site_name}</description>
    <lastBuildDate>{now}</lastBuildDate>
{items}  </channel>
</rss>"#
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
