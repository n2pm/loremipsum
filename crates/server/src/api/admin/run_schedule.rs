use std::{
    fs::File,
    io::{BufRead, BufReader},
    net::SocketAddr,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use common::api::{error::ErrorCodes, queue::get_work::QueueWork};
use serde::Deserialize;

use crate::{
    api::{auth::Auth, ErrorResponse, RouteError},
    db::register_task,
    state::ReqState,
};

#[derive(Debug, Deserialize)]
pub struct RunScheduleParams {
    pub schedule: String,
}

pub async fn run_schedule(
    State(state): ReqState,
    Query(params): Query<RunScheduleParams>,
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

    let schedule = state.config.schedules.get(&params.schedule);

    if schedule.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorCodes::InvalidSchedule as i32,
            }),
        ));
    }

    let schedule = schedule.unwrap();

    let schedule_file = File::open(schedule).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorCodes::BadScheduleFile as i32,
            }),
        )
    })?;

    let schedule_buf = BufReader::new(schedule_file);
    for ip in schedule_buf.lines().flatten() {
        let address: Result<SocketAddr, std::net::AddrParseError> = ip.parse();

        if let Ok(address) = address {
            let work = QueueWork::PingServer(address);
            register_task(work, &state.db).await.ok();
        }
    }

    Ok("done :3")
}
