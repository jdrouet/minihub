//! HTTP error response mapping.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use minihub_domain::error::MiniHubError;

/// JSON error body returned by API endpoints.
#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

/// Maps [`MiniHubError`] to an HTTP response with appropriate status code.
pub struct ApiError(MiniHubError);

impl From<MiniHubError> for ApiError {
    fn from(err: MiniHubError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            MiniHubError::Validation(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            MiniHubError::NotFound(err) => (StatusCode::NOT_FOUND, err.to_string()),
            MiniHubError::Storage(err) => {
                tracing::error!(error = %err, "storage error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
