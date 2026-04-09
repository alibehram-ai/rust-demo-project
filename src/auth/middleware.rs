use crate::auth::jwt::{validate_token, Claims};
use crate::error::AppError;
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use std::future::{ready, Ready};

pub struct AuthUser(pub Claims);

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match token {
            Some(token) => match validate_token(token) {
                Ok(claims) => ready(Ok(AuthUser(claims))),
                Err(e) => ready(Err(e)),
            },
            None => ready(Err(AppError::AuthError("No token provided".to_string()))),
        }
    }
}
