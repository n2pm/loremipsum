use common::api::queue::submit_work::{PlayerInfo, ServerInfo, ServerPingError};
use craftping::Chat;
use std::net::SocketAddr;
use tokio::net::TcpStream;

fn build_motd(motd: Chat) -> String {
    let mut string = String::new();
    string.push_str(&motd.text);

    for e in motd.extra {
        string.push_str(&build_motd(e));
    }

    string
}

pub async fn ping_server(server: SocketAddr) -> Result<ServerInfo, ServerPingError> {
    let mut tcp_stream = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        TcpStream::connect(server),
    )
    .await
    .map_err(|_| ServerPingError::ConnectionFailure)?
    .map_err(|_| ServerPingError::ConnectionFailure)?;

    let pong = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        craftping::tokio::ping(
            &mut tcp_stream,
            server.ip().to_string().as_str(),
            server.port(),
        ),
    )
    .await
    .map_err(|_| ServerPingError::PingFailure)?
    .map_err(|_| ServerPingError::PingFailure)?;

    let motd = build_motd(pong.description);

    Ok(ServerInfo {
        motd,

        version_string: pong.version,
        protocol_version: pong.protocol,
        connected_players: pong
            .sample
            .unwrap_or_default()
            .into_iter()
            .map(|p| PlayerInfo {
                name: p.name,
                uuid: p.id,
            })
            .collect(),
        online_player_count: pong.online_players as i32,
        max_player_count: pong.max_players as i32,
        favicon: pong.favicon,
    })
}
