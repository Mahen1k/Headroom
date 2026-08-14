use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;

use crate::auth::{hash_password, issue_token, verify_password};
use crate::error::AppError;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest};

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if req.email.trim().is_empty() || req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "email is required and password must be at least 8 characters".into(),
        ));
    }

    let hash = hash_password(&req.password)?;
    let role = "customer";

    let result = sqlx::query("INSERT INTO users (email, password_hash, role) VALUES (?, ?, ?)")
        .bind(&req.email)
        .bind(&hash)
        .bind(role)
        .execute(&pool)
        .await;

    let user_id = match result {
        Ok(r) => r.last_insert_rowid(),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err(AppError::BadRequest("email already registered".into()));
        }
        Err(e) => return Err(e.into()),
    };

    let token = issue_token(user_id, &req.email, role)?;
    Ok(Json(AuthResponse {
        token,
        email: req.email,
        role: role.into(),
    }))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row: Option<(i64, String, String)> =
        sqlx::query_as("SELECT id, password_hash, role FROM users WHERE email = ?")
            .bind(&req.email)
            .fetch_optional(&pool)
            .await?;

    let Some((user_id, hash, role)) = row else {
        return Err(AppError::Unauthorized("invalid email or password".into()));
    };

    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized("invalid email or password".into()));
    }

    let token = issue_token(user_id, &req.email, &role)?;
    Ok(Json(AuthResponse {
        token,
        email: req.email,
        role,
    }))
}
