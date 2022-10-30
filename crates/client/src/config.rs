use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub work: WorkConfig,
    pub api: ApiConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorkConfig {
    pub batch_size: usize,
    pub interval: usize,
    pub ipv6: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiConfig {
    pub address: String,
    pub api_key: String,
}
