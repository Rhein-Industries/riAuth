use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub struct Error {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
    pub fn bad(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request", message)
    }
    pub fn oauth(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }
    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "Authentication required or session expired",
        )
    }
    pub fn unauthenticated() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "authentication_required",
            "Bearer authentication required",
        )
    }
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN, "access_denied", "Access denied")
    }
    pub fn missing(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
    }
    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "operation failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Internal server error",
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = (
            self.status,
            Json(json!({"error": self.code, "error_description": self.message})),
        )
            .into_response();
        if self.status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                "www-authenticate",
                if self.code == "invalid_dpop_proof" {
                    "DPoP error=\"invalid_dpop_proof\"".parse().unwrap()
                } else if self.code == "invalid_client" {
                    "Basic realm=\"riauth\"".parse().unwrap()
                } else if self.code == "invalid_token" {
                    "Bearer realm=\"riauth\", error=\"invalid_token\""
                        .parse()
                        .unwrap()
                } else {
                    "Bearer realm=\"riauth\"".parse().unwrap()
                },
            );
        }
        if self.code == "insufficient_scope" {
            response.headers_mut().insert(
                "www-authenticate",
                "Bearer realm=\"riauth\", error=\"insufficient_scope\""
                    .parse()
                    .unwrap(),
            );
        }
        response
    }
}
