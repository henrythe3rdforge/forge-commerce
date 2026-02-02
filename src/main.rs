use std::sync::Arc;
use tera::Tera;

#[tokio::main]
async fn main() {
    let database = forge_commerce::db::init_db();
    let tera = Arc::new(Tera::new("templates/**/*.html").expect("Failed to load templates"));

    let app = forge_commerce::build_router((database, tera));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("🔨 Forge Commerce running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    println!("\n🛑 Shutting down gracefully...");
}
