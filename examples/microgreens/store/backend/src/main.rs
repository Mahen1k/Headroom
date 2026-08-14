use microgreens_store_backend::{build_router, db};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://microgreens.db?mode=rwc".into());
    let pool = db::init_pool(&database_url).await;
    let app = build_router(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("microgreens store backend listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
