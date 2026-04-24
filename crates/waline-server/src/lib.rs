mod handlers;
mod routes;
mod state;
pub mod middleware;
mod antispam;
mod avatar;
mod geo_ua;
mod rss;
mod webhook;
mod i18n;
mod security;
mod hooks;

use std::sync::Arc;
use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use waline_core::config::{Config, DatabaseType};
use waline_db::adapter::DatabaseAdapter;
use waline_common::error::WalineError;

use crate::state::AppState;

/// Wrapper for WalineError that implements IntoResponse
struct AppError(WalineError);

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self.0 {
            WalineError::NotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.0.to_string()),
            WalineError::Forbidden(_) => (axum::http::StatusCode::FORBIDDEN, self.0.to_string()),
            WalineError::BadRequest(_) => (axum::http::StatusCode::BAD_REQUEST, self.0.to_string()),
            WalineError::RateLimited => (axum::http::StatusCode::TOO_MANY_REQUESTS, self.0.to_string()),
            WalineError::SpamDetected => (axum::http::StatusCode::BAD_REQUEST, "Spam detected".to_string()),
            WalineError::Auth(_) => (axum::http::StatusCode::UNAUTHORIZED, self.0.to_string()),
            _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };
        (status, axum::Json(serde_json::json!({ "errmsg": message }))).into_response()
    }
}

impl From<WalineError> for AppError {
    fn from(e: WalineError) -> Self {
        AppError(e)
    }
}

/// Result type for handlers
type HandlerResult<T> = std::result::Result<T, AppError>;

pub async fn app() -> anyhow::Result<Router> {
    // Load configuration
    let config = Config::from_env().map_err(|e| anyhow::anyhow!(e))?;

    // Initialize database
    let db: Arc<dyn DatabaseAdapter> = match config.db_type {
        DatabaseType::Postgresql => {
            let url = config.postgres_url.clone().unwrap_or_else(|| {
                let host = config.pg_host.as_deref().unwrap_or("localhost");
                let port = config.pg_port.unwrap_or(5432);
                let user = config.pg_user.as_deref().unwrap_or("postgres");
                let password = config.pg_password.as_deref().unwrap_or("");
                let db_name = config.pg_db.as_deref().unwrap_or("waline");
                let ssl = config.pg_ssl.unwrap_or(false);
                let ssl_mode = if ssl { "require" } else { "prefer" };
                format!("postgres://{user}:{password}@{host}:{port}/{db_name}?sslmode={ssl_mode}")
            });
            Arc::new(
                waline_db::pg::PostgresAdapter::new(&url, config.table_prefix()).await?
            )
        }
        DatabaseType::Mysql => {
            Arc::new(
                waline_db::mysql::MysqlAdapter::new(
                    config.mysql_db.as_deref().unwrap_or("waline"),
                    config.mysql_host.as_deref().unwrap_or("localhost"),
                    config.mysql_port.unwrap_or(3306),
                    config.mysql_user.as_deref().unwrap_or("root"),
                    config.mysql_password.as_deref().unwrap_or(""),
                    config.mysql_charset.as_deref(),
                    config.mysql_ssl.unwrap_or(false),
                    config.table_prefix(),
                )
                .await?
            )
        }
        DatabaseType::Sqlite => {
            Arc::new(
                waline_db::sqlite::SqliteAdapter::new(
                    config.sqlite_path.as_deref().unwrap_or("waline.db"),
                    config.table_prefix(),
                )
                .await?
            )
        }
    };

    let state = Arc::new(AppState::new(config, db));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::create_router(state)
        .layer(RequestBodyLimitLayer::new(5 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::middleware::from_fn(middleware::version_header));

    Ok(app)
}
