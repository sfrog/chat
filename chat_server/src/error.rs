use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtError(_) => StatusCode::FORBIDDEN,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
