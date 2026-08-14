mod auth;
mod db;
mod error;
mod handlers;
mod models;

use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://microgreens.db?mode=rwc".into());
    let pool = db::init_pool(&database_url).await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/products", get(handlers::products::list_products))
        .route("/api/varieties", get(handlers::products::list_varieties))
        .route("/api/cart", get(handlers::cart::get_cart))
        .route("/api/cart/items", post(handlers::cart::add_to_cart))
        .route("/api/cart/items/:id", delete(handlers::cart::remove_from_cart))
        .route("/api/checkout", post(handlers::orders::checkout))
        .route("/api/orders", get(handlers::orders::list_orders))
        .route("/api/admin/batches", post(handlers::admin::add_batch))
        .route(
            "/api/admin/batches/:id/inspect",
            post(handlers::admin::inspect_batch),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("microgreens store backend listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
