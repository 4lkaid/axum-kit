use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Return `401 Unauthorized`
    #[error("Unauthorized")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("Forbidden")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("Not Found")]
    NotFound,

    /// Return
    /// - `400 Bad Request`
    /// - `415 Unsupported Media Type`
    /// - `422 Unprocessable Entity`
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),

    /// Return `422 Unprocessable Entity`
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    /// Return `500 Internal Server Error`
    #[cfg(feature = "redis")]
    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    /// Return
    /// - `404 Not Found` (Database Record Not Found)
    /// - `409 Conflict` (Unique Constraint Violation)
    /// - `500 Internal Server Error`
    #[cfg(feature = "postgres")]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error`
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

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

            #[cfg(feature = "redis")]
            Self::Redis(_) => internal_server_error(self),

            #[cfg(feature = "postgres")]
            Self::Sqlx(ref error) => match error {
                sqlx::Error::RowNotFound => (
                    StatusCode::NOT_FOUND,
                    "Database Record Not Found".to_string(),
                ),
                sqlx::Error::Database(db_error)
                    if db_error.code().is_some_and(|code| code == "23505") =>
                {
                    (
                        StatusCode::CONFLICT,
                        "Unique Constraint Violation".to_string(),
                    )
                }
                _ => internal_server_error(self),
            },

            Self::Anyhow(_) => internal_server_error(self),
            Self::Custom(statue, _) => (statue, self.to_string()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

fn internal_server_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    tracing::error!("{}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    )
}
