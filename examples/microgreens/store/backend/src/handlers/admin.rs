use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::auth::{require_admin, CurrentUser};
use crate::error::AppError;
use crate::models::{variety_window, AddBatchRequest, InspectRequest};

pub async fn add_batch(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Json(req): Json<AddBatchRequest>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;

    if variety_window(&req.variety).is_none() {
        return Err(AppError::BadRequest(format!(
            "unknown variety '{}'",
            req.variety
        )));
    }
    if req.quantity_trays <= 0 || req.price_per_tray < 0.0 {
        return Err(AppError::BadRequest(
            "quantity_trays must be positive and price_per_tray must not be negative".into(),
        ));
    }
    chrono::NaiveDate::parse_from_str(&req.sow_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("sow_date must be YYYY-MM-DD".into()))?;

    let result = sqlx::query(
        "INSERT INTO batches (variety, sow_date, quantity_trays, price_per_tray) VALUES (?, ?, ?, ?)",
    )
    .bind(&req.variety)
    .bind(&req.sow_date)
    .bind(req.quantity_trays)
    .bind(req.price_per_tray)
    .execute(&pool)
    .await?;

    Ok(Json(json!({ "id": result.last_insert_rowid() })))
}

pub async fn inspect_batch(
    State(pool): State<SqlitePool>,
    user: CurrentUser,
    Path(batch_id): Path<i64>,
    Json(req): Json<InspectRequest>,
) -> Result<Json<Value>, AppError> {
    require_admin(&user)?;

    let result = sqlx::query(
        "UPDATE batches SET height_cm = ?, mold = ?, color_uniform = ?, stem_ok = ?, root_mat_ok = ?
         WHERE id = ?",
    )
    .bind(req.height_cm)
    .bind(req.mold as i64)
    .bind(req.color_uniform as i64)
    .bind(req.stem_ok as i64)
    .bind(req.root_mat_ok as i64)
    .bind(batch_id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("no batch with id {batch_id}")));
    }

    Ok(Json(json!({ "updated": true })))
}
