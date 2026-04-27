use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Config(String),
    Database(sea_orm::DbErr),
    Io(std::io::Error),
    Unauthorized,
    Forbidden,
    NotFound,
    UserNotFound,
    UserRegistered,
    DuplicateContent,
    FrequencyLimited,
    TokenExpired,
    TwoFactorAuth,
    Akismet,
    Internal(String),
}

impl AppError {
    pub fn errno(&self) -> i32 {
        match self {
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            _ => 1000,
        }
    }

    pub fn message_key(&self) -> &str {
        match self {
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "FORBIDDEN",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::UserRegistered => "USER_REGISTERED",
            Self::DuplicateContent => "Duplicate Content",
            Self::FrequencyLimited => "Comment too fast!",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::TwoFactorAuth => "TWO_FACTOR_AUTH_ERROR_DETAIL",
            _ => "",
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message_key())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({
            "errno": self.errno(),
            "errmsg": self.message_key(),
        });
        (StatusCode::OK, axum::Json(body)).into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(e: sea_orm::DbErr) -> Self {
        tracing::error!("DB error: {e}");
        AppError::Database(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<envy::Error> for AppError {
    fn from(e: envy::Error) -> Self {
        AppError::Config(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        tracing::warn!("JWT error: {e}");
        AppError::Unauthorized
    }
}
