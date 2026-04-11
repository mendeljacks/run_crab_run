use rcr_core::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub struct ApiError(pub Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            Error::JobNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            Error::RunNotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            Error::InvalidRrule(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            Error::InvalidCommand(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            Error::WebhookSecretMismatch => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            Error::JobAlreadyRunning(_) => (StatusCode::CONFLICT, self.0.to_string()),
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
            Error::Execution(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
            Error::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
            Error::Other(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        (status, axum::Json(json!({ "error": message }))).into_response()
    }
}

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError(e)
    }
}