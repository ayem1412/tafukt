use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread::sleep;
use std::time::Duration;

use reqwest::{Client, Url, header};
use tokio::sync::mpsc;

use crate::protocol::Bencode;
use crate::protocol::decoder::Decoder;
use crate::tracker::error::TrackerError;
use crate::tracker::response::TrackerSuccessResponse;

mod error;
mod response;

const BITTORRENT_MIME_TYPE: &str = "application/x-bittorrent";

#[allow(clippy::too_many_arguments)]
fn build_url(
    announce_url: &str,
    info_hash: &[u8; 20],
    peer_id: &[u8; 20],
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: &str,
) -> Result<Url, TrackerError> {
    // reqwest can't parse binary values :(((((
    // https://github.com/seanmonstar/reqwest/issues/1613
    let peer_id = urlencoding::encode_binary(peer_id);
    let info_hash = urlencoding::encode_binary(info_hash);

    let base_url = format!("{}?peer_id={peer_id}&info_hash={info_hash}", announce_url);

    Url::parse_with_params(
        &base_url,
        &[
            ("port", port.to_string()),
            ("uploaded", uploaded.to_string()),
            ("downloaded", downloaded.to_string()),
            ("left", left.to_string()),
            ("compact", 1.to_string()),
            ("event", event.to_string()),
        ],
    )
    .map_err(|err| TrackerError::UrlParse(err.to_string()))
}

#[allow(clippy::too_many_arguments)]
async fn announce(
    client: &reqwest::Client,
    announce_url: &str,
    info_hash: &[u8; 20],
    peer_id: &[u8; 20],
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: &str,
) -> anyhow::Result<TrackerSuccessResponse> {
    let url = build_url(announce_url, info_hash, peer_id, 6881, uploaded, downloaded, left, event)?;
    let res = client.get(url).header(header::CONTENT_TYPE, BITTORRENT_MIME_TYPE).send().await?.bytes().await?;

    decode_response(&res)
}

pub async fn announce_loop(
    announce_url: &str,
    info_hash: &[u8; 20],
    peer_id: &[u8; 20],
    left: u64,
    peers_tx: mpsc::Sender<Vec<SocketAddr>>,
) {
    let mut event = "started".to_string();
    let client = Client::new();

    loop {
        match announce(&client, announce_url, info_hash, peer_id, 0, 0, left, &event).await {
            Ok(response) => {
                tracing::debug!(
                    "Tracker responded: {} peers, next announce in {}s",
                    response.peers.len(),
                    response.interval
                );

                if peers_tx.send(response.peers).await.is_err() {
                    tracing::error!("Tracker loop: engine receiver dropped, stopping.");
                    break;
                }
                event = String::new();
                sleep(Duration::from_secs(response.interval));
            },
            Err(err) => {
                tracing::error!("Tracker announce failed: {err}. Retrying in 30s.");
                sleep(Duration::from_secs(30));
            },
        }
    }
}

fn decode_response(bytes: &[u8]) -> anyhow::Result<TrackerSuccessResponse> {
    let mut iter = bytes.iter().copied();
    let mut decoder = Decoder::new(&mut iter);

    let bencode = decoder.decode().map_err(TrackerError::Bencode)?;

    let dict = match bencode {
        Bencode::Dictionary(dict) => dict,
        _ => anyhow::bail!("expected `dict`"),
    };

    if let Some(Bencode::String(reason)) = dict.get("failure reason") {
        anyhow::bail!("failure reason: {}", String::from_utf8_lossy(reason))
    }

    let interval = match dict.get("interval") {
        Some(Bencode::Integer(value)) => *value as u64,
        Some(_) => anyhow::bail!("expected `dict`"),
        None => anyhow::bail!("missing `interval` from response"),
    };

    let peers = match dict.get("peers") {
        Some(Bencode::String(bytes)) => bytes
            .chunks_exact(6)
            .map(|chunk| {
                let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                let port = u16::from_be_bytes([chunk[4], chunk[5]]);

                SocketAddr::V4(SocketAddrV4::new(ip, port))
            })
            .collect(),
        Some(_) => anyhow::bail!("expected `string`"),
        None => anyhow::bail!("missing `peers` from response"),
    };

    Ok(TrackerSuccessResponse::new(interval, peers))
}
