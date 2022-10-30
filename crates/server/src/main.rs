use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router, Server,
};
use common::logging;
use config::Config;
use log::{debug, info};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::db::register_task;

mod api;
mod config;
mod db;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup()?;

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());
    let config: Config = toml::from_str(tokio::fs::read_to_string(config_path).await?.as_str())?;

    debug!("config: {:#?}", config);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    tokio::task::spawn(async {
        tokio::signal::ctrl_c().await.ok();
        info!("caught c-c");
        shutdown_tx.send(()).ok();
    });

    info!("connecting to database");
    let options = PgConnectOptions::new()
        .host(&config.database.host)
        .port(config.database.port)
        .username(&config.database.username)
        .password(&config.database.password)
        .database(&config.database.database);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    info!("running migrations");
    sqlx::migrate!().run(&pool).await?;

    db::check_and_make_admin_key(&pool).await?;

    let state = Arc::new(state::State {
        config: config.clone(),
        db: pool,
        in_progress_work_queue: tokio::sync::Mutex::new(Vec::new()),
    });

    let work_queue_manager_state = Arc::clone(&state);
    tokio::task::spawn(async move {
        loop {
            let mut in_progress_work_queue =
                work_queue_manager_state.in_progress_work_queue.lock().await;

            let expired_work = in_progress_work_queue.clone();
            let expired_work = expired_work
                .iter()
                .filter(|x| x.timeout_by < chrono::Utc::now())
                .collect::<Vec<_>>();

            for work_status in expired_work {
                let pos_in_queue = in_progress_work_queue
                    .clone()
                    .iter()
                    .position(|x| x.work == work_status.work)
                    .unwrap();

                info!("removing expired work from queue: {:#?}", work_status);
                in_progress_work_queue.remove(pos_in_queue);
                register_task(work_status.work.clone(), &work_queue_manager_state.db)
                    .await
                    .unwrap();
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let router = Router::with_state(Arc::clone(&state))
        .route("/queue/get_work", get(api::queue::get_work))
        .route("/queue/submit_work", post(api::queue::submit_work))
        .route("/admin/run_schedule", post(api::admin::run_schedule))
        .route("/admin/gen_key", get(api::admin::gen_key));

    info!("starting server: listening on {}", &config.api.address);
    Server::bind(&config.api.address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await?;

    Ok(())
}
