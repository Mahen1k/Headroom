use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::models::{AddToCartRequest, CartItemView};

pub async fn get_cart(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
) -> Result<Json<Vec<CartItemView>>, AppError> {
    Ok(Json(load_cart(&pool, user.0.sub).await?))
}

pub async fn add_to_cart(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Json(req): Json<AddToCartRequest>,
) -> Result<Json<Vec<CartItemView>>, AppError> {
    if req.quantity <= 0 {
        return Err(AppError::BadRequest("quantity must be positive".into()));
    }

    let batch: Option<(i64,)> =
        sqlx::query_as("SELECT quantity_trays FROM batches WHERE id = ?")
            .bind(req.batch_id)
            .fetch_optional(&pool)
            .await?;
    let Some((available,)) = batch else {
        return Err(AppError::NotFound("batch not found".into()));
    };
    if req.quantity > available {
        return Err(AppError::BadRequest(format!(
            "only {available} tray(s) available"
        )));
    }

    sqlx::query(
        "INSERT INTO cart_items (user_id, batch_id, quantity) VALUES (?, ?, ?)
         ON CONFLICT(user_id, batch_id) DO UPDATE SET quantity = excluded.quantity",
    )
    .bind(user.0.sub)
    .bind(req.batch_id)
    .bind(req.quantity)
    .execute(&pool)
    .await?;

    Ok(Json(load_cart(&pool, user.0.sub).await?))
}

pub async fn remove_from_cart(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Path(item_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("DELETE FROM cart_items WHERE id = ? AND user_id = ?")
        .bind(item_id)
        .bind(user.0.sub)
        .execute(&pool)
        .await?;
    Ok(Json(json!({ "removed": true })))
}

pub async fn load_cart(pool: &SqlitePool, user_id: i64) -> Result<Vec<CartItemView>, AppError> {
    let rows: Vec<(i64, i64, String, i64, f64)> = sqlx::query_as(
        "SELECT ci.id, ci.batch_id, b.variety, ci.quantity, b.price_per_tray
         FROM cart_items ci JOIN batches b ON b.id = ci.batch_id
         WHERE ci.user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, batch_id, variety, quantity, unit_price)| CartItemView {
            id,
            batch_id,
            variety,
            quantity,
            unit_price,
            line_total: (quantity as f64) * unit_price,
        })
        .collect())
}
