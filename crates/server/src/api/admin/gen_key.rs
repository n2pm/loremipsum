use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::api::error::ErrorCodes;

use crate::{
    api::{auth::Auth, make_route_error, ErrorResponse, RouteError},
    state::ReqState,
};

pub async fn gen_key(
    State(state): ReqState,
    Auth(auth_state): Auth,
) -> Result<impl IntoResponse, RouteError> {
    if !auth_state.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorCodes::NotAdmin as i32,
            }),
        ));
    }

    let key = uuid::Uuid::new_v4().to_string();

    sqlx::query!("insert into api_keys (key, admin) values ($1, false)", key)
        .execute(&state.db)
        .await
        .map_err(|_| {
            make_route_error(StatusCode::INTERNAL_SERVER_ERROR, ErrorCodes::DatabaseError)
        })?;

    Ok(Json(key))
}
