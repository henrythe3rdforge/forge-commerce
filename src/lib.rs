pub mod auth;
pub mod db;
pub mod models;
pub mod routes;

use axum::{routing::{get, post, put}, Router};
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

pub fn build_router(state: (db::Db, Arc<Tera>)) -> Router {
    Router::new()
        // Pages
        .route("/", get(routes::products::index))
        .route("/products", get(routes::products::product_list))
        .route("/products/search", get(routes::products::product_list_partial))
        .route("/products/{id}", get(routes::products::product_detail))
        .route("/cart", get(routes::cart::view_cart))
        .route("/login", get(routes::auth::login_page).post(routes::auth::login))
        .route("/register", get(routes::auth::register_page).post(routes::auth::register))
        .route("/logout", get(routes::auth::logout))
        .route("/profile", get(routes::auth::profile))
        .route("/checkout", get(routes::checkout::checkout_page))
        .route("/checkout/shipping", post(routes::checkout::checkout_shipping))
        .route("/checkout/payment", post(routes::checkout::checkout_payment))
        .route("/checkout/confirm", post(routes::checkout::checkout_confirm))
        // API
        .route("/api/cart/add", post(routes::cart::add_to_cart))
        .route("/api/cart/{product_id}", put(routes::cart::update_cart).delete(routes::cart::remove_from_cart))
        .route("/api/cart/count", get(routes::cart::cart_count))
        // Admin
        .route("/admin", get(routes::admin::dashboard))
        .route("/admin/products", get(routes::admin::product_list))
        .route("/admin/products/new", get(routes::admin::product_new_page).post(routes::admin::product_create))
        .route("/admin/products/{id}/edit", get(routes::admin::product_edit_page))
        .route("/admin/products/{id}", post(routes::admin::product_update).delete(routes::admin::product_delete))
        .route("/admin/orders", get(routes::admin::order_list))
        .route("/admin/orders/{id}/status", post(routes::admin::order_update_status))
        // Health
        .route("/health", get(health))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}
