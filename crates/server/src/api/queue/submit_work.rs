use crate::{
    api::{auth::Auth, ErrorResponse, RouteError},
    state::ReqState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::api::{
    error::ErrorCodes,
    queue::{get_work::QueueWork, submit_work::WorkSubmit},
};
use log::debug;

pub async fn submit_work(
    State(state): ReqState,
    Auth(_auth_state): Auth,
    Json(work): Json<Vec<WorkSubmit>>,
) -> Result<impl IntoResponse, RouteError> {
    //debug!("got work: {:#?}", work);

    let mut in_progress_work_queue = state.in_progress_work_queue.lock().await;

    for submit in work {
        match submit {
            WorkSubmit::PingServer((server, result)) => {
                debug!("got ping result for {}: {:?}", server, result);
                let work_in_queue = in_progress_work_queue
                    .iter()
                    .position(|x| x.work == QueueWork::PingServer(server));

                if work_in_queue.is_none() {
                    debug!("got ping result for server that was not in progress");

                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: ErrorCodes::UnknownWork as i32,
                        }),
                    ));
                } else {
                    debug!("removing ping result from in progress queue");
                    in_progress_work_queue.remove(work_in_queue.unwrap());

                    if let Some(result) = result {
                        debug!("pushing ping result to database");
                        let id = format!("{}:{}", server.ip(), server.port());

                        sqlx::query!(
                            r#"
                        insert into seen_servers (id)
                        values ($1)
                        on conflict do nothing
                        "#,
                            id
                        )
                        .execute(&state.db)
                        .await
                        .ok();

                        sqlx::query!(
                            r#"
                        insert into seen_servers_data
                        (time, id, motd, favicon, current_player_count,
                        max_player_count, server_version, protocol_version)
                        values (now(), $1, $2, $3, $4, $5, $6, $7)
                        "#,
                            id,
                            result.motd,
                            result.favicon,
                            result.online_player_count,
                            result.max_player_count,
                            result.version_string,
                            result.protocol_version
                        )
                        .execute(&state.db)
                        .await
                        .ok();

                        for player in result.connected_players {
                            debug!("pushing player result to database");

                            sqlx::query!(
                                r#"
                            insert into seen_players (id)
                            values ($1)
                            on conflict do nothing
                            "#,
                                player.uuid
                            )
                            .execute(&state.db)
                            .await
                            .ok();

                            sqlx::query!(
                                r#"
                                insert into seen_players_data (time, id, name, connected_to)
                                values (now(), $1, $2, $3)"#,
                                player.uuid,
                                player.name,
                                id
                            )
                            .execute(&state.db)
                            .await
                            .ok();
                        }
                    }
                }
            }
            WorkSubmit::GetPlayer((uuid, name)) => {
                debug!("got player result for {}: {:?}", uuid, name);
            }
        }
    }

    Ok("thanks :3")
}
