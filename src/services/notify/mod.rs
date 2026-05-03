pub mod email;
pub mod discord;
pub mod lark;
pub mod pushplus;
pub mod qq;
pub mod serverchan;
pub mod telegram;
pub mod wechat;

use crate::config::Config;
use crate::models::comment;
use crate::models::user;
use serde_json::{Value, json};

pub struct NotifyContext<'a> {
    pub config: &'a Config,
    pub comment: &'a comment::Model,
    #[allow(dead_code)]
    pub comment_user: Option<&'a user::Model>,
    pub parent: Option<&'a comment::Model>,
    #[allow(dead_code)]
    pub parent_user: Option<&'a user::Model>,
    pub raw_comment: &'a str,
}

impl<'a> NotifyContext<'a> {
    pub fn post_url(&self) -> String {
        let site_url = self.config.site_url.as_deref().unwrap_or("");
        let url = self.comment.url.as_deref().unwrap_or("");
        format!("{site_url}{url}#{}", self.comment.id)
    }

    pub fn site_name(&self) -> &str {
        self.config.site_name.as_deref().unwrap_or("")
    }

    pub fn nick(&self) -> &str {
        self.comment.nick.as_deref().unwrap_or("")
    }

    pub fn mail(&self) -> &str {
        self.comment.mail.as_deref().unwrap_or("")
    }

    pub fn comment_text(&self) -> &str {
        self.raw_comment
    }

    pub fn template_data(&self) -> Value {
        json!({
            "self": {
                "nick": self.nick(),
                "mail": self.mail(),
                "comment": self.comment_text(),
                "url": self.comment.url,
                "objectId": self.comment.id,
                "status": self.comment.status,
            },
            "parent": self.parent.map(|p| json!({
                "nick": p.nick,
                "mail": p.mail,
                "comment": p.comment,
                "url": p.url,
                "objectId": p.id,
            })),
            "site": {
                "name": self.site_name(),
                "url": self.config.site_url,
                "postUrl": self.post_url(),
            }
        })
    }
}

/// Run all configured notification channels.
pub async fn run(ctx: &NotifyContext<'_>, disable_author_notify: bool) {
    let author_email = ctx.config.author_email.as_deref().unwrap_or("");
    let comment_mail = ctx.comment.mail.as_deref().unwrap_or("");
    let is_author_comment = !author_email.is_empty()
        && comment_mail.to_lowercase() == author_email.to_lowercase();

    // Notify blog author (via push channels or email)
    if !ctx.config.disable_author_notify && !is_author_comment && !disable_author_notify {
        let mut notified_via_push = false;

        if serverchan::send(ctx).await { notified_via_push = true; }
        if wechat::send(ctx).await { notified_via_push = true; }
        if qq::send(ctx).await { notified_via_push = true; }
        if telegram::send(ctx).await { notified_via_push = true; }
        if pushplus::send(ctx).await { notified_via_push = true; }
        if discord::send(ctx).await { notified_via_push = true; }
        if lark::send(ctx).await { notified_via_push = true; }

        // Fall back to email if no push channel notified
        if !notified_via_push && !author_email.is_empty() {
            let subject = ctx.config.mail_subject_admin.as_deref()
                .unwrap_or("MAIL_SUBJECT_ADMIN");
            let template = ctx.config.mail_template_admin.as_deref()
                .unwrap_or("MAIL_TEMPLATE_ADMIN");
            email::send_mail(ctx.config, author_email, subject, template, ctx.template_data()).await;
        }
    }

    // Notify parent comment author (reply notification)
    if let Some(parent) = ctx.parent {
        let parent_mail = parent.mail.as_deref().unwrap_or("");
        let is_self_reply = comment_mail.to_lowercase() == parent_mail.to_lowercase();
        let is_reply_to_author = !author_email.is_empty()
            && parent_mail.to_lowercase() == author_email.to_lowercase();

        // Skip fake OAuth emails like xxx@mail.github
        let is_fake_mail = parent_mail.contains("@mail.");

        if !is_self_reply && !is_reply_to_author && !is_fake_mail && !parent_mail.is_empty()
            && ctx.comment.status != "waiting"
        {
            let subject = ctx.config.mail_subject.as_deref().unwrap_or("MAIL_SUBJECT");
            let template = ctx.config.mail_template.as_deref().unwrap_or("MAIL_TEMPLATE");
            email::send_mail(ctx.config, parent_mail, subject, template, ctx.template_data()).await;
        }
    }
}
