use axum::extract::State;
use axum::Json;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{evaluate_salability, known_varieties, BatchRow, Product};

pub async fn list_varieties() -> Json<Vec<&'static str>> {
    Json(known_varieties())
}

pub async fn list_products(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Product>>, AppError> {
    let rows: Vec<BatchRow> = sqlx::query_as(
        "SELECT id, variety, sow_date, quantity_trays, price_per_tray, height_cm,
                mold, color_uniform, stem_ok, root_mat_ok
         FROM batches",
    )
    .fetch_all(&pool)
    .await?;

    let today = Utc::now().date_naive();
    let products = rows
        .into_iter()
        .filter_map(|row| {
            let (salable, grade, _) = evaluate_salability(&row, today);
            if !salable || row.quantity_trays <= 0 {
                return None;
            }
            let sow_date = chrono::NaiveDate::parse_from_str(&row.sow_date, "%Y-%m-%d").ok()?;
            Some(Product {
                id: row.id,
                variety: row.variety,
                sow_date: row.sow_date.clone(),
                days_since_sowing: (today - sow_date).num_days(),
                quantity_trays: row.quantity_trays,
                price_per_tray: row.price_per_tray,
                grade,
            })
        })
        .collect();

    Ok(Json(products))
}
