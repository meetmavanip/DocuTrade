use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use crate::errors::AppError;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;

use crate::services::auth::Claims;

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| {
            if header_str.starts_with("Bearer ") {
                Some(header_str[7..].to_string())
            } else {
                None
            }
        });

    let token = auth_header.ok_or_else(|| AppError::Auth("Missing authorization header".into()))?;

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "super_secret_jwt_key_for_dev_only".into());
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ).map_err(|_| AppError::Auth("Invalid token".into()))?;

    req.extensions_mut().insert(token_data.claims);
    
    Ok(next.run(req).await)
}
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct RequireBuyer;

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for RequireBuyer {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<Claims>()
            .ok_or_else(|| AppError::Auth("User not authenticated".into()))?;

        if claims.role.to_uppercase() != "BUYER" {
            return Err(AppError::Auth("Access forbidden: Requires BUYER role".into()));
        }
        Ok(RequireBuyer)
    }
}

pub struct RequireSeller;

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for RequireSeller {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<Claims>()
            .ok_or_else(|| AppError::Auth("User not authenticated".into()))?;

        if claims.role.to_uppercase() != "SELLER" {
            return Err(AppError::Auth("Access forbidden: Requires SELLER role".into()));
        }
        Ok(RequireSeller)
    }
}
