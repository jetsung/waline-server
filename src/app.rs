use std::net::SocketAddr;

use axum::middleware;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{
    config::Config,
    db,
    error::AppError,
    geoip::{self, GeoIp},
    middleware::{cors, secure_domains},
    routes,
    state::AppState,
};

pub async fn start(config: Config) -> Result<(), AppError> {
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool, &config.database_url).await?;

    // Initialize GeoIP
    let geoip_config = config.geoip_config();
    let geoip_instance = geoip::create_geoip(&geoip_config).await;

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse().unwrap();
    let state = AppState::new(pool, config, geoip_instance);

    let app = routes::build(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), secure_domains))
        .layer(middleware::from_fn(cors))
        .layer(TraceLayer::new_for_http())
        .into_make_service_with_connect_info::<SocketAddr>();

    info!("Waline server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
