use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use microgreens_store_backend::{build_router, db};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn test_app() -> Router {
    let pool = db::init_pool("sqlite::memory:").await;
    build_router(pool)
}

async fn request(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, body)
}

fn json_post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn json_post_auth(uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn get_auth(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

async fn register_and_login(app: &Router, email: &str) -> String {
    let (status, body) = request(
        app,
        json_post(
            "/api/auth/register",
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "register failed: {body:?}");
    body["token"].as_str().unwrap().to_string()
}

async fn promote_to_admin(pool: &SqlitePool, email: &str) {
    sqlx::query("UPDATE users SET role = 'admin' WHERE email = ?")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn health_check_ok() {
    let app = test_app().await;
    let response = app.oneshot(get("/api/health")).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(&bytes[..], b"ok");
}

#[tokio::test]
async fn register_then_duplicate_email_is_rejected() {
    let app = test_app().await;
    let _ = register_and_login(&app, "dup@store.test").await;

    let (status, body) = request(
        &app,
        json_post(
            "/api/auth/register",
            json!({ "email": "dup@store.test", "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("already registered"));
}

#[tokio::test]
async fn login_with_wrong_password_is_unauthorized() {
    let app = test_app().await;
    register_and_login(&app, "wrongpw@store.test").await;

    let (status, _) = request(
        &app,
        json_post(
            "/api/auth/login",
            json!({ "email": "wrongpw@store.test", "password": "not-the-password" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn products_list_excludes_unrated_and_immature_batches() {
    let pool = db::init_pool("sqlite::memory:").await;
    let app = build_router(pool.clone());

    register_and_login(&app, "admin@store.test").await;
    promote_to_admin(&pool, "admin@store.test").await;
    // Re-login so the issued JWT carries the promoted admin role.
    let admin_token = {
        let (_, body) = request(
            &app,
            json_post(
                "/api/auth/login",
                json!({ "email": "admin@store.test", "password": "password123" }),
            ),
        )
        .await;
        body["token"].as_str().unwrap().to_string()
    };

    // Batch just added has no quality check yet -> should not appear in the catalog.
    let (status, body) = request(
        &app,
        json_post_auth(
            "/api/admin/batches",
            &admin_token,
            json!({
                "variety": "sunflower",
                "sow_date": "2026-08-04",
                "quantity_trays": 10,
                "price_per_tray": 4.5
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let batch_id = body["id"].as_i64().unwrap();

    let (_, products) = request(&app, get("/api/products")).await;
    assert!(products.as_array().unwrap().is_empty());

    let (status, _) = request(
        &app,
        json_post_auth(
            &format!("/api/admin/batches/{batch_id}/inspect"),
            &admin_token,
            json!({
                "height_cm": 7.5,
                "mold": false,
                "color_uniform": true,
                "stem_ok": true,
                "root_mat_ok": true
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, products) = request(&app, get("/api/products")).await;
    let products = products.as_array().unwrap();
    assert_eq!(products.len(), 1);
    assert_eq!(products[0]["grade"], "A");
}

#[tokio::test]
async fn non_admin_cannot_add_batch() {
    let app = test_app().await;
    let token = register_and_login(&app, "customer@store.test").await;

    let (status, _) = request(
        &app,
        json_post_auth(
            "/api/admin/batches",
            &token,
            json!({
                "variety": "sunflower",
                "sow_date": "2026-08-04",
                "quantity_trays": 10,
                "price_per_tray": 4.5
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn full_purchase_flow_with_discount_decrements_stock() {
    let pool = db::init_pool("sqlite::memory:").await;
    let app = build_router(pool.clone());

    register_and_login(&app, "admin2@store.test").await;
    promote_to_admin(&pool, "admin2@store.test").await;
    let (_, body) = request(
        &app,
        json_post(
            "/api/auth/login",
            json!({ "email": "admin2@store.test", "password": "password123" }),
        ),
    )
    .await;
    let admin_token = body["token"].as_str().unwrap().to_string();

    let (_, body) = request(
        &app,
        json_post_auth(
            "/api/admin/batches",
            &admin_token,
            json!({
                "variety": "sunflower",
                "sow_date": "2026-08-04",
                "quantity_trays": 10,
                "price_per_tray": 4.5
            }),
        ),
    )
    .await;
    let batch_id = body["id"].as_i64().unwrap();

    request(
        &app,
        json_post_auth(
            &format!("/api/admin/batches/{batch_id}/inspect"),
            &admin_token,
            json!({
                "height_cm": 7.5,
                "mold": false,
                "color_uniform": true,
                "stem_ok": true,
                "root_mat_ok": true
            }),
        ),
    )
    .await;

    let buyer_token = register_and_login(&app, "buyer@store.test").await;

    let (status, _) = request(
        &app,
        json_post_auth(
            "/api/cart/items",
            &buyer_token,
            json!({ "batch_id": batch_id, "quantity": 3 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, order) = request(
        &app,
        json_post_auth(
            "/api/checkout",
            &buyer_token,
            json!({ "discount_code": "FRESH10" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(order["subtotal"], 13.5);
    assert_eq!(order["total"], 12.15);
    assert_eq!(order["discount_code"], "FRESH10");

    let (_, orders) = request(&app, get_auth("/api/orders", &buyer_token)).await;
    assert_eq!(orders.as_array().unwrap().len(), 1);

    let (_, products) = request(&app, get("/api/products")).await;
    assert_eq!(products[0]["quantity_trays"], 7);
}

#[tokio::test]
async fn checkout_with_empty_cart_is_rejected() {
    let app = test_app().await;
    let token = register_and_login(&app, "emptycart@store.test").await;

    let (status, body) = request(
        &app,
        json_post_auth("/api/checkout", &token, json!({ "discount_code": null })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("empty"));
}

#[tokio::test]
async fn checkout_with_invalid_discount_code_is_rejected() {
    let pool = db::init_pool("sqlite::memory:").await;
    let app = build_router(pool.clone());

    register_and_login(&app, "admin3@store.test").await;
    promote_to_admin(&pool, "admin3@store.test").await;
    let (_, body) = request(
        &app,
        json_post(
            "/api/auth/login",
            json!({ "email": "admin3@store.test", "password": "password123" }),
        ),
    )
    .await;
    let admin_token = body["token"].as_str().unwrap().to_string();

    let (_, body) = request(
        &app,
        json_post_auth(
            "/api/admin/batches",
            &admin_token,
            json!({
                "variety": "radish",
                "sow_date": "2026-08-08",
                "quantity_trays": 5,
                "price_per_tray": 3.0
            }),
        ),
    )
    .await;
    let batch_id = body["id"].as_i64().unwrap();
    request(
        &app,
        json_post_auth(
            &format!("/api/admin/batches/{batch_id}/inspect"),
            &admin_token,
            json!({
                "height_cm": 6.0,
                "mold": false,
                "color_uniform": true,
                "stem_ok": true,
                "root_mat_ok": true
            }),
        ),
    )
    .await;

    let buyer_token = register_and_login(&app, "buyer2@store.test").await;
    request(
        &app,
        json_post_auth(
            "/api/cart/items",
            &buyer_token,
            json!({ "batch_id": batch_id, "quantity": 1 }),
        ),
    )
    .await;

    let (status, body) = request(
        &app,
        json_post_auth(
            "/api/checkout",
            &buyer_token,
            json!({ "discount_code": "NOTREAL" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("invalid"));
}

#[tokio::test]
async fn protected_route_without_token_is_unauthorized() {
    let app = test_app().await;
    let (status, _) = request(&app, get("/api/cart")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
