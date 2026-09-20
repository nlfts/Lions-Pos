use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{errors::AppError, models::user::Claims, state::AppState};

pub fn create_jwt(
    user_id: &str,
    username: &str,
    roles: Vec<String>,
    perms: Vec<String>,
    secret: &str,
    expiration_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let exp = now + (expiration_hours as usize * 3600);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        roles,
        perms,
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub Claims);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Token otentikasi tidak ditemukan".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Format header otentikasi tidak valid".to_string()))?;

        let claims = decode_jwt(token, &state.config.jwt_secret)
            .map_err(|e| AppError::Unauthorized(format!("Token tidak valid atau kadaluarsa: {}", e)))?;

        Ok(AuthenticatedUser(claims))
    }
}

#[derive(Clone, Debug)]
pub struct OptionalUser(pub Option<Claims>);

impl FromRequestParts<AppState> for OptionalUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = match parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
        {
            Some(h) => h,
            None => return Ok(OptionalUser(None)),
        };

        let token = match auth_header.strip_prefix("Bearer ") {
            Some(t) => t,
            None => return Ok(OptionalUser(None)),
        };

        match decode_jwt(token, &state.config.jwt_secret) {
            Ok(claims) => Ok(OptionalUser(Some(claims))),
            Err(_) => Ok(OptionalUser(None)),
        }
    }
}
