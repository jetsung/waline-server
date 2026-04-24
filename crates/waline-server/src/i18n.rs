use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde_json::Value;

/// Supported languages
const SUPPORTED_LANGS: &[&str] = &[
    "zh-CN", "zh-TW", "en", "pt-BR", "ru", "fr", "de", "ja", "es", "ko", "id", "tr",
];

/// Simple i18n translation system
/// Stores translations as key -> {lang -> value}
static TRANSLATIONS: Lazy<HashMap<&'static str, HashMap<&'static str, &'static str>>> =
    Lazy::new(|| {
        let mut m = HashMap::new();

        // Error messages
        let mut rate_limit = HashMap::new();
        rate_limit.insert("zh-CN", "评论太频繁，请稍后再试");
        rate_limit.insert("en", "Comment too frequently, please try again later");
        m.insert("rate_limit", rate_limit);

        let mut spam = HashMap::new();
        spam.insert("zh-CN", "评论被识别为垃圾评论");
        spam.insert("en", "Comment identified as spam");
        m.insert("spam_detected", spam);

        let mut auth_required = HashMap::new();
        auth_required.insert("zh-CN", "请先登录");
        auth_required.insert("en", "Please login first");
        m.insert("auth_required", auth_required);

        let mut email_exists = HashMap::new();
        email_exists.insert("zh-CN", "邮箱已被注册");
        email_exists.insert("en", "Email already registered");
        m.insert("email_exists", email_exists);

        let mut invalid_password = HashMap::new();
        invalid_password.insert("zh-CN", "密码错误");
        invalid_password.insert("en", "Invalid password");
        m.insert("invalid_password", invalid_password);

        let mut user_not_found = HashMap::new();
        user_not_found.insert("zh-CN", "用户不存在");
        user_not_found.insert("en", "User not found");
        m.insert("user_not_found", user_not_found);

        let mut user_banned = HashMap::new();
        user_banned.insert("zh-CN", "用户已被封禁");
        user_banned.insert("en", "User is banned");
        m.insert("user_banned", user_banned);

        let mut twofa_required = HashMap::new();
        twofa_required.insert("zh-CN", "请输入两步验证码");
        twofa_required.insert("en", "2FA code required");
        m.insert("2fa_required", twofa_required);

        let mut twofa_invalid = HashMap::new();
        twofa_invalid.insert("zh-CN", "两步验证码错误");
        twofa_invalid.insert("en", "Invalid 2FA code");
        m.insert("2fa_invalid", twofa_invalid);

        let mut forbidden_word = HashMap::new();
        forbidden_word.insert("zh-CN", "评论包含违禁词");
        forbidden_word.insert("en", "Comment contains forbidden words");
        m.insert("forbidden_word", forbidden_word);

        m
    });

/// Detect language from Accept-Language header or lang query parameter
pub fn detect_lang(accept_language: Option<&str>, lang_param: Option<&str>) -> String {
    // Priority: lang query parameter > Accept-Language header > "en"
    if let Some(lang) = lang_param {
        if SUPPORTED_LANGS.contains(&lang.as_ref()) {
            return lang.to_string();
        }
    }

    if let Some(al) = accept_language {
        // Parse Accept-Language: take the first language
        if let Some(first) = al.split(',').next() {
            let lang = first.split(';').next().unwrap_or("en").trim();
            // Normalize: zh-CN, zh-TW, en, etc.
            let normalized = match lang {
                "zh" | "zh-CN" | "zh-Hans" => "zh-CN",
                "zh-TW" | "zh-Hant" => "zh-TW",
                l if l.starts_with("en") => "en",
                l if l.starts_with("pt") => "pt-BR",
                l if l.starts_with("ru") => "ru",
                l if l.starts_with("fr") => "fr",
                l if l.starts_with("de") => "de",
                l if l.starts_with("ja") => "ja",
                l if l.starts_with("es") => "es",
                l if l.starts_with("ko") => "ko",
                l if l.starts_with("id") => "id",
                l if l.starts_with("tr") => "tr",
                _ => "en",
            };
            return normalized.to_string();
        }
    }

    "en".to_string()
}

/// Get a translated message
pub fn t(key: &str, lang: &str) -> String {
    TRANSLATIONS
        .get(key)
        .and_then(|translations| translations.get(lang))
        .or_else(|| TRANSLATIONS.get(key).and_then(|t| t.get("en")))
        .unwrap_or(&key)
        .to_string()
}
