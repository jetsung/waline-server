use axum::{
    routing::{delete, get, post, put},
    Router,
};
use crate::state::AppState;
use crate::handlers;
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        // Comment API
        .route("/comment", get(handlers::comment::get_comments))
        .route("/comment", post(handlers::comment::add_comment))
        .route("/comment/{id}", put(handlers::comment::update_comment))
        .route("/comment/{id}", delete(handlers::comment::delete_comment))
        // Article/Counter API
        .route("/article", get(handlers::article::get_article))
        .route("/article", post(handlers::article::update_article))
        // Token/Auth API
        .route("/token", get(handlers::token::get_token))
        .route("/token", post(handlers::token::login))
        .route("/token", delete(handlers::token::logout))
        // 2FA API
        .route("/token/2fa", get(handlers::twofa::get_2fa_setup))
        .route("/token/2fa", post(handlers::twofa::enable_2fa))
        // User API
        .route("/user", get(handlers::user::get_users))
        .route("/user", post(handlers::user::register))
        .route("/user", put(handlers::user::update_user))
        .route("/user/{id}", delete(handlers::user::delete_user))
        // Password Reset
        .route("/user/password", put(handlers::password::reset_password))
        // OAuth API
        .route("/oauth", get(handlers::oauth::oauth))
        // Database Management API
        .route("/db", get(handlers::db::export_db))
        .route("/db", post(handlers::db::import_db))
        .route("/db", put(handlers::db::update_db))
        .route("/db", delete(handlers::db::clear_db))
        // Verification API
        .route("/verification", get(handlers::verification::verify_email))
        .with_state(state.clone());

    // Dashboard (admin UI) - catch-all for /ui/* paths
    let ui = Router::new()
        .route("/", get(handlers::dashboard::dashboard))
        .route("/*path", get(handlers::dashboard::dashboard))
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .nest("/ui", ui)
        .route("/", get(handlers::root::root))
}
