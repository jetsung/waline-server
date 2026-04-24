use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalineError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Spam detected")]
    SpamDetected,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("Markdown rendering error: {0}")]
    Markdown(String),

    #[error("Config error: {0}")]
    Config(String),
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for WalineError {
    fn from(e: sqlx::Error) -> Self {
        WalineError::Database(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, WalineError>;
