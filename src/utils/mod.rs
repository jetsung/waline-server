pub mod avatar;
pub mod ip;
pub mod jwt;
pub mod markdown;
pub mod password;
pub mod spam;
pub mod ua;

use axum::http::{HeaderMap, header};

pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

pub fn extract_ip(headers: &HeaderMap, fallback: &str) -> String {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

#[allow(dead_code)]
pub fn extract_lang(query_lang: Option<&str>, headers: &HeaderMap) -> String {
    if let Some(lang) = query_lang {
        if !lang.is_empty() {
            return lang.to_string();
        }
    }
    headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "en".to_string())
}
