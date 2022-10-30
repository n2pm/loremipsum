use crate::{
    api::{auth::Auth, make_route_error, RouteError},
    db::get_tasks,
    state::ReqState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use common::api::{error::ErrorCodes, queue::get_work::WorkStatus};
use serde::Deserialize;

// todo lock behind api key
pub async fn get_work(
    State(state): ReqState,
    Query(params): Query<WorkParams>,
    Auth(_auth_state): Auth,
) -> Result<impl IntoResponse, RouteError> {
    let mut work_vec = Vec::new();
    let mut in_progress_work_queue = state.in_progress_work_queue.lock().await;

    let tasks = get_tasks(params.batch_size, &state.db).await.map_err(|_| {
        make_route_error(StatusCode::INTERNAL_SERVER_ERROR, ErrorCodes::DatabaseError)
    })?;

    for work in tasks {
        work_vec.push(work.clone());
        in_progress_work_queue.push(WorkStatus {
            work,
            timeout_by: chrono::Utc::now() + chrono::Duration::seconds(30),
        });
    }

    Ok(Json(work_vec))
}

#[derive(Debug, Deserialize)]
pub struct WorkParams {
    pub batch_size: usize,
    pub ipv6: bool,
}
