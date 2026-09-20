use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
    Conflict(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, err_code) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, "BAD_REQUEST"),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, "NOT_FOUND"),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, "UNAUTHORIZED"),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg, "FORBIDDEN"),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, "INTERNAL_ERROR"),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg, "CONFLICT"),
        };

        let body = Json(ErrorResponse {
            success: false,
            message: message.clone(),
            error: Some(err_code.to_string()),
        });

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Data tidak ditemukan".to_string()),
            _ => AppError::Internal(format!("Kesalahan database: {}", err)),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized(format!("Token tidak valid: {}", err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Internal(format!("Kesalahan enkripsi password: {}", err))
    }
}
