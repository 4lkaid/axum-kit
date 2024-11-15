use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Return `401 Unauthorized`
    #[allow(dead_code)]
    #[error("Unauthorized")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[allow(dead_code)]
    #[error("Forbidden")]
    Forbidden,

    /// Return `404 Not Found`
    #[allow(dead_code)]
    #[error("Not Found")]
    NotFound,

    /// Return
    /// * `400 Bad Request`
    /// * `415 Unsupported Media Type`
    /// * `422 Unprocessable Entity`
    #[allow(dead_code)]
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),

    /// Return `422 Unprocessable Entity`
    #[allow(dead_code)]
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    /// Redis, Sqlx and Anyhow
    /// Return `500 Internal Server Error`
    #[allow(dead_code)]
    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[allow(dead_code)]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[allow(dead_code)]
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[allow(dead_code)]
    #[error("{1}")]
    Custom(StatusCode, String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let (status, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::JsonExtractorRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
            Self::ValidationError(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::Redis(_) | Self::Sqlx(_) | Self::Anyhow(_) => {
                tracing::error!("{}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            Self::Custom(statue, _) => (statue, self.to_string()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
