use crate::error::AppError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-change-in-production".to_string())
});

static JWT_EXPIRATION_HOURS: Lazy<i64> = Lazy::new(|| {
    env::var("JWT_EXPIRATION_HOURS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24)
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(user_id: &str, email: &str) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let expiration = now + chrono::Duration::hours(*JWT_EXPIRATION_HOURS);

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(AppError::from)
}

pub fn validate_token(token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map_err(AppError::from)?;

    Ok(token_data.claims)
}
