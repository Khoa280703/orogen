use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NoAccounts,
    GrokApi(String),
    Internal(String),
    BadRequest(String),
    #[allow(dead_code)]
    Forbidden(String),
    NotFound(String),
    #[allow(dead_code)]
    Unauthorized,
    ModelNotAllowed,
    QuotaExceeded(String),
    PlanRequired,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAccounts => write!(f, "No upstream accounts configured"),
            Self::GrokApi(msg) => write!(f, "Grok API error: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::BadRequest(msg) => write!(f, "{msg}"),
            Self::Forbidden(msg) => write!(f, "{msg}"),
            Self::NotFound(msg) => write!(f, "{msg}"),
            Self::Unauthorized => write!(f, "Invalid API token"),
            Self::ModelNotAllowed => write!(f, "Model not available in your plan"),
            Self::QuotaExceeded(message) => write!(f, "{message}"),
            Self::PlanRequired => write!(f, "Active plan required"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::NoAccounts => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service unavailable".to_string(),
            ),
            Self::GrokApi(_) => (
                StatusCode::BAD_GATEWAY,
                "External service error".to_string(),
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message.clone()),
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, message.clone()),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            Self::ModelNotAllowed => (
                StatusCode::FORBIDDEN,
                "Model not available in your plan".to_string(),
            ),
            Self::QuotaExceeded(message) => (StatusCode::TOO_MANY_REQUESTS, message.clone()),
            Self::PlanRequired => (StatusCode::FORBIDDEN, "Active plan required".to_string()),
        };
        // In debug mode, include detailed error for development
        #[cfg(debug_assertions)]
        let response = if cfg!(debug_assertions) {
            (
                status,
                Json(json!({ "error": { "message": message, "debug": self.to_string() } })),
            )
                .into_response()
        } else {
            (status, Json(json!({ "error": { "message": message } }))).into_response()
        };

        #[cfg(not(debug_assertions))]
        let response = (status, Json(json!({ "error": { "message": message } }))).into_response();

        response
    }
}
