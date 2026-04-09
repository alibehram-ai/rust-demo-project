use crate::auth::jwt::create_token;
use crate::db::DbPool;
use crate::error::AppError;
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};
use actix_web::{post, web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "User already exists"),
    ),
    tag = "auth"
)]
#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(&body.email)
        .fetch_one(pool.get_ref())
        .await?;

    if existing > 0 {
        return Err(AppError::UserAlreadyExists);
    }

    let id = Uuid::new_v4().to_string();
    let password_hash = hash(&body.password, DEFAULT_COST).map_err(AppError::from)?;
    let created_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.name)
    .bind(&created_at)
    .execute(pool.get_ref())
    .await?;

    let token = create_token(&id, &body.email)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: UserResponse {
            id,
            email: body.email.clone(),
            name: body.name.clone(),
            created_at,
        },
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "auth"
)]
#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let valid = verify(&body.password, &user.password_hash).map_err(AppError::from)?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let token = create_token(&user.id, &user.email)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    }))
}
