use std::collections::HashMap;
use once_cell::sync::Lazy;

static TRANSLATIONS: Lazy<HashMap<&'static str, HashMap<&'static str, &'static str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    let mut en = HashMap::new();
    en.insert("MAIL_SUBJECT_ADMIN", "New comment on {{site.name}}");
    en.insert("MAIL_TEMPLATE_ADMIN", r#"<p>Hi,</p><p>{{comment.nick}} commented on <a href="{{site.postUrl}}">{{site.name}}</a>:</p><blockquote>{{comment.comment}}</blockquote><p><a href="{{site.postUrl}}">View comment</a></p>"#);
    en.insert("MAIL_SUBJECT", "{{comment.nick}} replied to your comment on {{site.name}}");
    en.insert("MAIL_TEMPLATE", r#"<p>Hi,</p><p>{{comment.nick}} replied to your comment on <a href="{{site.postUrl}}">{{site.name}}</a>:</p><blockquote>{{comment.comment}}</blockquote><p><a href="{{site.postUrl}}">View comment</a></p>"#);
    en.insert("Registration Confirm Mail", "Confirm registration on {{name}}");
    en.insert("confirm registration", r#"<p>Please click the link below to confirm your registration:</p><p><a href="{{url}}">{{url}}</a></p>"#);
    en.insert("Reset Password", "Reset password for {{name}}");
    en.insert("Please click link to login and change your password as soon as possible!", r#"<p>Please click the link below to reset your password:</p><p><a href="{{url}}">{{url}}</a></p>"#);
    en.insert("Duplicate Content", "Duplicate Content");
    en.insert("Comment too fast!", "Comment too fast!");
    en.insert("USER_REGISTERED", "User already registered");
    en.insert("USER_NOT_FOUND", "User not found");
    en.insert("Unauthorized", "Unauthorized");
    en.insert("FORBIDDEN", "Forbidden");
    en.insert("TOKEN_EXPIRED", "Token expired");
    en.insert("TWO_FACTOR_AUTH_ERROR_DETAIL", "Two-factor authentication failed");
    map.insert("en", en);

    let mut zh_cn = HashMap::new();
    zh_cn.insert("MAIL_SUBJECT_ADMIN", "{{site.name}} 有新评论啦");
    zh_cn.insert("MAIL_TEMPLATE_ADMIN", r#"<p>您好，</p><p>{{comment.nick}} 在 <a href="{{site.postUrl}}">{{site.name}}</a> 发表了新评论：</p><blockquote>{{comment.comment}}</blockquote><p><a href="{{site.postUrl}}">查看评论</a></p>"#);
    zh_cn.insert("MAIL_SUBJECT", "{{comment.nick}} 回复了您在 {{site.name}} 的评论");
    zh_cn.insert("MAIL_TEMPLATE", r#"<p>您好，</p><p>{{comment.nick}} 回复了您在 <a href="{{site.postUrl}}">{{site.name}}</a> 的评论：</p><blockquote>{{comment.comment}}</blockquote><p><a href="{{site.postUrl}}">查看评论</a></p>"#);
    zh_cn.insert("Registration Confirm Mail", "确认注册 {{name}}");
    zh_cn.insert("confirm registration", r#"<p>请点击以下链接确认注册：</p><p><a href="{{url}}">{{url}}</a></p>"#);
    zh_cn.insert("Reset Password", "重置 {{name}} 密码");
    zh_cn.insert("Please click link to login and change your password as soon as possible!", r#"<p>请点击以下链接重置密码：</p><p><a href="{{url}}">{{url}}</a></p>"#);
    zh_cn.insert("Duplicate Content", "重复内容");
    zh_cn.insert("Comment too fast!", "评论太快了！");
    zh_cn.insert("USER_REGISTERED", "用户已注册");
    zh_cn.insert("USER_NOT_FOUND", "用户不存在");
    zh_cn.insert("Unauthorized", "未授权");
    zh_cn.insert("FORBIDDEN", "禁止访问");
    zh_cn.insert("TOKEN_EXPIRED", "令牌已过期");
    zh_cn.insert("TWO_FACTOR_AUTH_ERROR_DETAIL", "双因素认证失败");
    map.insert("zh-CN", zh_cn.clone());
    map.insert("zh", zh_cn);

    map
});

pub fn translate(lang: &str, key: &str) -> String {
    // Try exact match, then language prefix, then English
    let lang_map = TRANSLATIONS.get(lang)
        .or_else(|| {
            let prefix = lang.split('-').next().unwrap_or(lang);
            TRANSLATIONS.get(prefix)
        })
        .or_else(|| TRANSLATIONS.get("en"));

    lang_map
        .and_then(|m| m.get(key))
        .map(|s| s.to_string())
        .unwrap_or_else(|| key.to_string())
}
