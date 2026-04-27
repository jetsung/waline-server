use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    handlers::{article, comment, db, index, oauth, rss, ui, user, verification},
    state::AppState,
};

pub fn build(state: AppState) -> Router {
    let api = Router::new()
        // Comment — static routes BEFORE parameterized
        .route("/comment/rss", get(rss::rss_feed))
        .route("/comment", get(comment::get_comment))
        .route("/comment", post(comment::post_comment))
        .route("/comment/{id}", put(comment::put_comment))
        .route("/comment/{id}", delete(comment::delete_comment))
        // Article counter
        .route("/article", get(article::get_article))
        .route("/article", post(article::post_article))
        // Auth
        .route("/token", get(user::get_token))
        .route("/token", post(user::login))
        .route("/token", delete(user::logout))
        .route("/token/2fa", get(user::get_2fa))
        .route("/token/2fa", post(user::set_2fa))
        // User
        .route("/user", get(user::get_user))
        .route("/user", post(user::register))
        .route("/user", put(user::update_profile))
        .route("/user/password", put(user::reset_password))
        .route("/user/{id}", put(user::set_user_type))
        .route("/user/{id}", delete(user::delete_user))
        // OAuth (separate routes for redirect and callback)
        .route("/oauth", get(oauth::oauth_redirect))
        .route("/oauth/callback", get(oauth::oauth_callback))
        // DB management
        .route("/db", get(db::export_db))
        .route("/db", post(db::import_record))
        .route("/db", put(db::update_record))
        .route("/db", delete(db::delete_db))
        // Email verification
        .route("/verification", get(verification::verify_email));

    Router::new()
        .nest("/api", api)
        // Admin UI SPA — explicit routes matching waline-mini
        .route("/ui", get(ui::ui_page))
        .route("/ui/", get(ui::ui_page))
        .route("/ui/login", get(ui::ui_page))
        .route("/ui/register", get(ui::ui_page))
        .route("/ui/forgot", get(ui::ui_page))
        .route("/ui/profile", get(ui::ui_page))
        .route("/ui/user", get(ui::ui_page))
        .route("/ui/migration", get(ui::ui_page))
        // Root - Waline example page (same as Node.js version)
        .route("/", get(index::index_page))
        .with_state(state)
}
