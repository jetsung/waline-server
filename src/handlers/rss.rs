use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;

use crate::{models::comment, state::AppState, utils::markdown};

#[derive(Debug, Deserialize)]
pub struct RssQuery {
    pub path: Option<String>,
    pub email: Option<String>,
    pub user_id: Option<i64>,
    pub count: Option<i64>,
}

struct CommentRow {
    id: i64,
    nick: Option<String>,
    comment: Option<String>,
    url: Option<String>,
    inserted_at: Option<String>,
}

/// GET /api/comment/rss
pub async fn rss_feed(State(state): State<AppState>, Query(q): Query<RssQuery>) -> Response {
    let limit = q.count.unwrap_or(20).clamp(1, 50) as u64;
    let site_url = state.config.site_url.as_deref().unwrap_or("");
    let site_name = state.config.site_name.as_deref().unwrap_or("Waline");

    let comments = match fetch_comments(&state, &q, limit).await {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let items: Vec<String> = comments
        .iter()
        .map(|c| {
            let nick = c.nick.as_deref().unwrap_or("Anonymous");
            let comment_url = c.url.as_deref().unwrap_or("");
            let link = if comment_url.is_empty() {
                format!("{site_url}#{}", c.id)
            } else {
                format!("{site_url}{comment_url}#{}", c.id)
            };
            let description = markdown::render(c.comment.as_deref().unwrap_or(""));
            let title = format!(
                "{nick} commented{}",
                c.url.as_deref().map(|u| format!(" on {u}")).unwrap_or_default()
            );
            let pub_date = format_rfc2822(c.inserted_at.as_deref().unwrap_or(""));
            format!(
                "        <item>\n            <title><![CDATA[{title}]]></title>\n            <description><![CDATA[{description}]]></description>\n            <link>{link}</link>\n            <guid isPermaLink=\"false\">{}</guid>\n            <pubDate>{pub_date}</pubDate>\n        </item>",
                c.id
            )
        })
        .collect();

    let (title, desc) = if let Some(p) = &q.path {
        (
            format!("{site_name} Comments for {p}"),
            format!("Recent comments for {p}."),
        )
    } else if q.email.is_some() || q.user_id.is_some() {
        (
            format!("{site_name} Reply Comments"),
            "Recent reply comments.".to_string(),
        )
    } else {
        (
            format!("{site_name} Recent Comments"),
            "Recent comments.".to_string(),
        )
    };

    let now = format_rfc2822(&chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\" xmlns:atom=\"http://www.w3.org/2005/Atom\" version=\"2.0\">\n    <channel>\n        <title><![CDATA[{title}]]></title>\n        <description><![CDATA[{desc}]]></description>\n        <link>{site_url}</link>\n        <generator>RSS for Rust</generator>\n        <lastBuildDate>{now}</lastBuildDate>\n        <pubDate>{now}</pubDate>\n{}\n    </channel>\n</rss>",
        items.join("\n")
    );

    (
        StatusCode::OK,
        [("Content-Type", "application/rss+xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

fn format_rfc2822(datetime_str: &str) -> String {
    use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};

    let dt = if let Ok(ndt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
        Utc.from_utc_datetime(&ndt)
    } else if let Ok(dt) = DateTime::parse_from_rfc3339(datetime_str) {
        dt.with_timezone(&Utc)
    } else {
        Utc::now()
    };

    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

async fn fetch_comments(
    state: &AppState,
    q: &RssQuery,
    limit: u64,
) -> Result<Vec<CommentRow>, sea_orm::DbErr> {
    if let Some(path) = &q.path {
        let models = comment::Entity::find()
            .filter(comment::Column::Url.eq(path))
            .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
            .order_by_desc(comment::Column::InsertedAt)
            .limit(limit)
            .all(&state.db)
            .await?;
        return Ok(models
            .into_iter()
            .map(|m| CommentRow {
                id: m.id,
                nick: m.nick,
                comment: m.comment,
                url: m.url,
                inserted_at: m.inserted_at,
            })
            .collect());
    }

    if q.email.is_some() || q.user_id.is_some() {
        let parent_ids: Vec<i64> = if let Some(email) = &q.email {
            comment::Entity::find()
                .select_only()
                .column(comment::Column::Id)
                .filter(comment::Column::Mail.eq(email))
                .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
                .into_tuple::<i64>()
                .all(&state.db)
                .await?
        } else {
            comment::Entity::find()
                .select_only()
                .column(comment::Column::Id)
                .filter(comment::Column::UserId.eq(q.user_id.unwrap()))
                .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
                .into_tuple::<i64>()
                .all(&state.db)
                .await?
        };

        if parent_ids.is_empty() {
            return Ok(vec![]);
        }

        let models = comment::Entity::find()
            .filter(comment::Column::Pid.is_in(parent_ids))
            .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
            .order_by_desc(comment::Column::InsertedAt)
            .limit(limit)
            .all(&state.db)
            .await?;
        return Ok(models
            .into_iter()
            .map(|m| CommentRow {
                id: m.id,
                nick: m.nick,
                comment: m.comment,
                url: m.url,
                inserted_at: m.inserted_at,
            })
            .collect());
    }

    let models = comment::Entity::find()
        .filter(comment::Column::Status.is_not_in(["waiting", "spam"]))
        .order_by_desc(comment::Column::InsertedAt)
        .limit(limit)
        .all(&state.db)
        .await?;
    Ok(models
        .into_iter()
        .map(|m| CommentRow {
            id: m.id,
            nick: m.nick,
            comment: m.comment,
            url: m.url,
            inserted_at: m.inserted_at,
        })
        .collect())
}
