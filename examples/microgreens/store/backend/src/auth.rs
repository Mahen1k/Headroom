use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

const JWT_SECRET: &[u8] = b"microgreens-store-dev-secret-change-me";
const TOKEN_TTL_SECS: i64 = 60 * 60 * 24; // 24h

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub email: String,
    pub role: String,
    pub exp: i64,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("hashing failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn issue_token(user_id: i64, email: &str, role: &str) -> Result<String, AppError> {
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: chrono::Utc::now().timestamp() + TOKEN_TTL_SECS,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| AppError::Internal(format!("token signing failed: {e}")))
}

/// Extracts and verifies a Bearer JWT, making the claims available to handlers
/// via `CurrentUser` in the function signature.
pub struct CurrentUser(pub Claims);

#[axum::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("expected Bearer token".into()))?;

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))?;

        Ok(CurrentUser(data.claims))
    }
}

pub fn require_admin(user: &CurrentUser) -> Result<(), AppError> {
    if user.0.role != "admin" {
        return Err(AppError::Forbidden("admin access required".into()));
    }
    Ok(())
}
