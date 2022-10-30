use std::sync::{Arc, Mutex};

use common::{
    api::queue::{get_work::QueueWork, submit_work::WorkSubmit},
    logging,
};
use config::Config;
use futures::AsyncReadExt;
use isahc::{AsyncReadResponseExt, RequestExt};
use log::{debug, info};

mod config;
mod ping;

async fn do_work(config: Config) -> anyhow::Result<()> {
    let base_addr = config.api.address;
    let work_config = config.work;

    let url = format!(
        "{}/queue/get_work?batch_size={}&ipv6={}",
        base_addr, work_config.batch_size, work_config.ipv6
    );
    let mut response = isahc::Request::get(url)
        .header("Authorization", config.api.api_key.clone())
        .body(())?
        .send_async()
        .await?;

    let work: Vec<QueueWork> = response.json().await?;
    let mut handles = Vec::new();

    let response_arcmutex = Arc::new(Mutex::new(Vec::new()));

    for job in work {
        let response_arcmutex = response_arcmutex.clone();

        let handle = tokio::spawn(async move {
            match job {
                QueueWork::PingServer(server) => {
                    info!("definitely pinging server {}", server);
                    let result = ping::ping_server(server).await;
                    let result = result.ok();

                    response_arcmutex
                        .lock()
                        .unwrap()
                        .push(WorkSubmit::PingServer((server, result)));
                }
                QueueWork::GetPlayer(uuid) => {
                    info!("definitely getting player {}", uuid);
                    // TODO
                    todo!();
                }
            }
        });

        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    // FIXME i have not seen god.
    let lmfao = response_arcmutex.lock().unwrap().clone();
    if lmfao.is_empty() {
        info!("no work to submit");
        return Ok(());
    }

    let url = format!("{}/queue/submit_work", base_addr);
    let body = serde_json::to_string(&lmfao)?;
    debug!("body: {}", body);
    let mut deez = isahc::Request::post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", config.api.api_key)
        .body(body)?
        .send_async()
        .await?;

    let mut lol = String::new();
    deez.body_mut().read_to_string(&mut lol).await?;

    debug!("response: {:#?}", lol);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup()?;

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());
    let config: Config = toml::from_str(tokio::fs::read_to_string(config_path).await?.as_str())?;

    debug!("{:#?}", config);

    loop {
        do_work(config.clone()).await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(
            config.work.interval.try_into().unwrap(),
        ))
        .await;
    }
}
