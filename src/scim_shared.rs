//! SCIM media-type formatting and schema identifiers shared with outbound provisioning.
#[cfg(feature = "platform")]
use crate::error::Result;
#[cfg(feature = "platform")]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(feature = "platform")]
use serde_json::{Value, json};

pub const USER: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
pub const GROUP: &str = "urn:ietf:params:scim:schemas:core:2.0:Group";

#[cfg(feature = "platform")]
pub fn response(result: Result<Value>, status: StatusCode) -> Response {
    let mut response = match result {
        Ok(value) => {
            let location = value["meta"]["location"].as_str().map(String::from);
            let etag = value["meta"]["version"].as_str().map(String::from);
            let mut out = if status == StatusCode::NO_CONTENT {
                status.into_response()
            } else {
                (status, Json(value)).into_response()
            };
            if let Some(location) = location.and_then(|v| v.parse().ok()) {
                out.headers_mut().insert("location", location);
            }
            if let Some(etag) = etag.and_then(|v| v.parse().ok()) {
                out.headers_mut().insert("etag", etag);
            }
            out
        }
        Err(mut error) => {
            if error.code == "conflict" && error.message == "Configuration revision changed" {
                error.status = StatusCode::PRECONDITION_FAILED;
                error.code = "precondition_failed";
            }
            let typ = match error.code {
                "conflict" => "uniqueness",
                "invalid_filter" => "invalidFilter",
                "mutability" => "mutability",
                _ => "invalidValue",
            };
            let mut body = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:Error"],"status":error.status.as_u16().to_string(),"detail":error.message});
            if error.code != "precondition_failed" {
                body["scimType"] = json!(typ);
            }
            (error.status, Json(body)).into_response()
        }
    };
    if status != StatusCode::NO_CONTENT || response.status() != StatusCode::NO_CONTENT {
        response
            .headers_mut()
            .insert("content-type", "application/scim+json".parse().unwrap());
    }
    response
}
