use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::handlers::cart::load_cart;
use crate::models::{CheckoutRequest, OrderView};

pub async fn checkout(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<OrderView>, AppError> {
    let items = load_cart(&pool, user.0.sub).await?;
    if items.is_empty() {
        return Err(AppError::BadRequest("cart is empty".into()));
    }

    // Re-validate stock against current batch quantities before committing the order.
    for item in &items {
        let available: Option<(i64,)> =
            sqlx::query_as("SELECT quantity_trays FROM batches WHERE id = ?")
                .bind(item.batch_id)
                .fetch_optional(&pool)
                .await?;
        match available {
            Some((qty,)) if qty >= item.quantity => {}
            _ => {
                return Err(AppError::BadRequest(format!(
                    "{} is no longer available in the requested quantity",
                    item.variety
                )))
            }
        }
    }

    let subtotal: f64 = items.iter().map(|i| i.line_total).sum();

    let mut discount_code = None;
    let mut total = subtotal;
    if let Some(code) = req.discount_code.as_ref().filter(|c| !c.trim().is_empty()) {
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT percent_off FROM discounts WHERE code = ? AND active = 1",
        )
        .bind(code)
        .fetch_optional(&pool)
        .await?;
        let Some((percent_off,)) = row else {
            return Err(AppError::BadRequest("invalid or inactive discount code".into()));
        };
        total = subtotal * (1.0 - percent_off / 100.0);
        discount_code = Some(code.clone());
    }
    total = (total * 100.0).round() / 100.0;

    let mut tx = pool.begin().await?;

    let order_result = sqlx::query(
        "INSERT INTO orders (user_id, discount_code, subtotal, total) VALUES (?, ?, ?, ?)",
    )
    .bind(user.0.sub)
    .bind(&discount_code)
    .bind(subtotal)
    .bind(total)
    .execute(&mut *tx)
    .await?;
    let order_id = order_result.last_insert_rowid();

    for item in &items {
        sqlx::query(
            "INSERT INTO order_items (order_id, batch_id, variety, quantity, unit_price)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(order_id)
        .bind(item.batch_id)
        .bind(&item.variety)
        .bind(item.quantity)
        .bind(item.unit_price)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE batches SET quantity_trays = quantity_trays - ? WHERE id = ?")
            .bind(item.quantity)
            .bind(item.batch_id)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("DELETE FROM cart_items WHERE user_id = ?")
        .bind(user.0.sub)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(OrderView {
        id: order_id,
        subtotal,
        total,
        discount_code,
        items,
    }))
}

pub async fn list_orders(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
) -> Result<Json<Vec<OrderView>>, AppError> {
    let orders: Vec<(i64, f64, f64, Option<String>)> = sqlx::query_as(
        "SELECT id, subtotal, total, discount_code FROM orders WHERE user_id = ? ORDER BY id DESC",
    )
    .bind(user.0.sub)
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::with_capacity(orders.len());
    for (id, subtotal, total, discount_code) in orders {
        let items: Vec<(i64, i64, String, i64, f64)> = sqlx::query_as(
            "SELECT id, batch_id, variety, quantity, unit_price FROM order_items WHERE order_id = ?",
        )
        .bind(id)
        .fetch_all(&pool)
        .await?;

        let items = items
            .into_iter()
            .map(
                |(item_id, batch_id, variety, quantity, unit_price)| crate::models::CartItemView {
                    id: item_id,
                    batch_id,
                    variety,
                    quantity,
                    unit_price,
                    line_total: (quantity as f64) * unit_price,
                },
            )
            .collect();

        result.push(OrderView {
            id,
            subtotal,
            total,
            discount_code,
            items,
        });
    }

    Ok(Json(result))
}
