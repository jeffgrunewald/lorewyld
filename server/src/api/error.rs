use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::error;

/// Unified API error type. Handlers return `Result<T, ApiError>`;
/// `ApiError` knows how to render itself as a JSON error response.
#[derive(Debug)]
pub enum ApiError {
    /// Registration failed because the supplied join code did not match
    /// the server's `join_code`.
    InvalidJoinCode,
    /// Registration failed because the display name is already taken on
    /// this server.
    DisplayNameTaken,
    /// Login failed because no user with the given display name exists.
    DisplayNameNotFound,
    /// Request lacked a valid `Authorization: Bearer <token>` header.
    Unauthorized,
    /// Requested resource does not exist.
    NotFound,
    /// Request body or path parameters were malformed beyond what serde
    /// caught.
    BadRequest(String),
    /// Catch-all for unexpected failures (database errors, etc.).
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        Self::Internal(value.into())
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::InvalidJoinCode => (
                StatusCode::FORBIDDEN,
                "invalid_join_code",
                "join code did not match this server".to_string(),
            ),
            Self::DisplayNameTaken => (
                StatusCode::CONFLICT,
                "display_name_taken",
                "display name is already in use on this server".to_string(),
            ),
            Self::DisplayNameNotFound => (
                StatusCode::NOT_FOUND,
                "display_name_not_found",
                "no user with that display name exists on this server".to_string(),
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid session token".to_string(),
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_string(),
            ),
            Self::BadRequest(detail) => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                detail.clone(),
            ),
            Self::Internal(err) => {
                error!(error = ?err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".to_string(),
                )
            }
        };

        (
            status,
            Json(ErrorBody {
                code,
                message: &message,
            }),
        )
            .into_response()
    }
}
