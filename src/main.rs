mod app;
mod config;
mod db;
mod error;
mod geoip;
mod handlers;
mod locales;
mod middleware;
mod models;
mod response;
mod routes;
mod services;
mod state;
mod utils;

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    let config = config::Config::load()?;

    let env_filter = if config.debug {
        EnvFilter::new("debug,sea_orm=debug,tower_http=debug")
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,sea_orm=warn,hyper=warn"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_timer(fmt::time::LocalTime::rfc_3339()))
        .with(env_filter)
        .init();

    app::start(config).await
}
