use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum QueueWork {
    PingServer(SocketAddr),
    GetPlayer(String),
}

#[derive(Debug, Clone)]
pub struct WorkStatus {
    pub work: QueueWork,
    pub timeout_by: chrono::DateTime<chrono::Utc>,
}
