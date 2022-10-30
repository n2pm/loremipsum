use axum::{http::StatusCode, Json};
use common::api::error::ErrorCodes;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: i32,
}

pub type RouteError = (StatusCode, Json<ErrorResponse>);

pub fn make_route_error(status_code: StatusCode, error_code: ErrorCodes) -> RouteError {
    (
        status_code,
        Json(ErrorResponse {
            error: error_code as i32,
        }),
    )
}

pub mod admin;
pub mod auth;
pub mod queue;
