use axum::{
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::response::Json as JsonResponse;

#[allow(dead_code)]
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
    TokenExpired,
    TwoFactorAuth,
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
        JsonResponse(body).into_response()
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
