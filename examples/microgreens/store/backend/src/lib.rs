pub mod auth;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_router(pool: SqlitePool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
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
        .with_state(pool)
}
