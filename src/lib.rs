pub mod auth;
pub mod db;
pub mod models;
pub mod routes;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

pub fn build_router(state: (db::Db, Arc<Tera>)) -> Router {
    Router::new()
        // Marketplace feed
        .route("/", get(routes::listings::feed))
        .route("/search", get(routes::listings::feed_partial))
        // Listings
        .route("/sell", get(routes::listings::new_listing_page).post(routes::listings::create_listing))
        .route("/listing/{id}", get(routes::listings::listing_detail))
        .route("/listing/{id}/edit", get(routes::listings::edit_listing_page).post(routes::listings::update_listing))
        .route("/listing/{id}/sold", post(routes::listings::mark_sold))
        .route("/listing/{id}/delete", post(routes::listings::delete_listing))
        // Messages
        .route("/messages", get(routes::messages::inbox))
        .route("/messages/{id}", get(routes::messages::conversation))
        .route("/messages/{id}/send", post(routes::messages::send_message))
        .route("/messages/{id}/offer", post(routes::messages::make_offer))
        .route("/messages/{convo_id}/offer/{offer_id}/respond", get(routes::messages::respond_offer))
        .route("/messages/{id}/poll", get(routes::messages::poll_messages))
        // Start conversation from listing
        .route("/listing/{id}/contact", get(routes::messages::start_conversation))
        // Auth
        .route("/login", get(routes::auth::login_page).post(routes::auth::login))
        .route("/register", get(routes::auth::register_page).post(routes::auth::register))
        .route("/logout", get(routes::auth::logout))
        .route("/profile", get(routes::auth::profile).post(routes::auth::update_profile))
        // Health
        .route("/health", get(health))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}
