use crate::config::Config;
use common::api::queue::get_work::WorkStatus;
use std::sync::Arc;

pub type ArcState = Arc<State>;
pub type ReqState = axum::extract::State<ArcState>;

pub struct State {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub in_progress_work_queue: tokio::sync::Mutex<Vec<WorkStatus>>,
}
