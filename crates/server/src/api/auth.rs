use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use common::api::error::ErrorCodes;

use crate::state::ArcState;

use super::{make_route_error, RouteError};

pub struct AuthState {
    pub api_key: String,
    pub admin: bool,
}

pub struct Auth(pub AuthState);

#[async_trait]
impl FromRequestParts<ArcState> for Auth {
    type Rejection = RouteError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ArcState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| make_route_error(StatusCode::UNAUTHORIZED, ErrorCodes::ApiKeyInvalid))?;

        let row = sqlx::query!(
            "select admin from api_keys where key = $1",
            header.to_str().unwrap()
        )
        .fetch_one(&state.db)
        .await
        .map_err(|_| {
            make_route_error(StatusCode::INTERNAL_SERVER_ERROR, ErrorCodes::DatabaseError)
        })?;

        return Ok(Auth(AuthState {
            api_key: header.to_str().unwrap().to_string(),
            admin: row.admin,
        }));
    }
}
