use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File storage error: {0}")]
    FileStorage(#[from] crate::services::file_storage::FileStorageError),

    #[error("IA client error: {0}")]
    IAClient(#[from] crate::services::ia_client::IAClientError),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    /// Get the HTTP status code for this error
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => StatusCode::FORBIDDEN,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Database(_)
            | AppError::FileStorage(_)
            | AppError::IAClient(_)
            | AppError::Jwt(_)
            | AppError::Serialization(_)
            | AppError::HttpClient(_)
            | AppError::Io(_)
            | AppError::Configuration(_)
            | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error category for logging
    pub fn category(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database",
            AppError::Authentication(_) => "auth",
            AppError::Authorization(_) => "authz",
            AppError::Validation(_) => "validation",
            AppError::FileStorage(_) => "file_storage",
            AppError::IAClient(_) => "ia_client",
            AppError::Jwt(_) => "jwt",
            AppError::Serialization(_) => "serialization",
            AppError::HttpClient(_) => "http_client",
            AppError::Io(_) => "io",
            AppError::Configuration(_) => "configuration",
            AppError::NotFound(_) => "not_found",
            AppError::Conflict(_) => "conflict",
            AppError::BadRequest(_) => "bad_request",
            AppError::Internal(_) => "internal",
        }
    }

    /// Check if error should be logged as warning vs error
    pub fn should_log_as_error(&self) -> bool {
        match self {
            AppError::Authentication(_)
            | AppError::Authorization(_)
            | AppError::Validation(_)
            | AppError::NotFound(_)
            | AppError::Conflict(_)
            | AppError::BadRequest(_) => false, // These are expected client errors
            _ => true, // Server errors should be logged as errors
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();

        // Log error if it's a server error
        if self.should_log_as_error() {
            tracing::error!(
                error = %self,
                category = self.category(),
                status = %status_code,
                "Application error occurred"
            );
        } else {
            tracing::warn!(
                error = %self,
                category = self.category(),
                status = %status_code,
                "Client error occurred"
            );
        }

        // Create error response
        let error_message = match status_code {
            StatusCode::INTERNAL_SERVER_ERROR => {
                // Don't expose internal error details to clients
                "An internal server error occurred".to_string()
            }
            _ => self.to_string(),
        };

        let body = json!({
            "success": false,
            "error": error_message,
            "code": status_code.as_u16(),
        });

        (status_code, Json(body)).into_response()
    }
}

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Helper trait for converting validation errors
pub trait ValidationExt<T> {
    fn validation_error(self, message: impl Into<String>) -> AppResult<T>;
}

impl<T, E> ValidationExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn validation_error(self, message: impl Into<String>) -> AppResult<T> {
        self.map_err(|e| AppError::Validation(format!("{}: {}", message.into(), e)))
    }
}

/// Helper macros for creating common errors
#[macro_export]
macro_rules! auth_error {
    ($msg:expr) => {
        $crate::error::AppError::Authentication($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::Authentication(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! authz_error {
    ($msg:expr) => {
        $crate::error::AppError::Authorization($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::Authorization(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::error::AppError::Validation($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::Validation(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! not_found {
    ($msg:expr) => {
        $crate::error::AppError::NotFound($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::NotFound(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! conflict_error {
    ($msg:expr) => {
        $crate::error::AppError::Conflict($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::Conflict(format!($fmt, $($arg)*))
    };
}
