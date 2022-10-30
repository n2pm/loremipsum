use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub schedules: HashMap<String, PathBuf>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiConfig {
    pub address: SocketAddr,
}
