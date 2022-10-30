use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WorkSubmit {
    PingServer((SocketAddr, Option<ServerInfo>)),
    GetPlayer((String, Option<String>)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub motd: String,

    pub version_string: String,
    pub protocol_version: i32,

    pub connected_players: Vec<PlayerInfo>,
    pub online_player_count: i32,
    pub max_player_count: i32,

    pub favicon: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: String,
}

#[derive(Error, Debug)]
pub enum ServerPingError {
    #[error("failed to connect to socket")]
    ConnectionFailure,
    #[error("failed to ping server")]
    PingFailure,
}
