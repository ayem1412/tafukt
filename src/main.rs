use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::{RwLock, mpsc};
use tokio::time::timeout;
use tracing::Level;

use crate::metainfo::Metainfo;
use crate::peer::handshake;
use crate::protocol::decoder::Decoder;

mod metainfo;
mod peer;
mod piece;
mod protocol;
mod tracker;
mod util;

const MAX_PEERS: u8 = 50;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    /* let invalid_string = unsafe {
        // archlinux-2026.03.01-x86_64.iso.torrent
        // 716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent
        // debian.iso.torrent
        String::from_utf8_unchecked(include_bytes!("../torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").to_vec())
    }; */

    /* let file = File::open("./torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").unwrap();
    let reader = BufReader::new(file);
    let mut bytes = reader.bytes().map(|c| c.unwrap()); */
    let file_content = std::fs::read("./torrents/debian.iso.torrent").unwrap();
    let mut bytes = file_content.iter().copied();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode().unwrap();

    let metainfo = Metainfo::try_from(result)?;

    let info = metainfo.info;

    let name = info.name.clone();
    let length = info.length();
    let piece_length = info.piece_length;
    let piece_count = info.piece_count();

    tracing::debug!("Torrent {name} ({piece_count} pieces x {piece_length} bytes = {length} bytes total)");

    let announce_url = metainfo.announce.unwrap();
    let info_hash = info.info_hash().as_ref().unwrap().as_ref().try_into().unwrap();
    let peer_id = util::generate_peer_id();

    let (peers_tx, mut peers_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        tracker::announce_loop(&announce_url, info_hash, &peer_id, length, peers_tx).await;
    });

    // let mut valid_peers = Arc::new(RwLock::new(HashSet::new()));
    let mut active_peers = 0;

    while let Some(peers) = peers_rx.recv().await {
        for peer in peers {
            tracing::debug!("Received peer: {peer}");

            if active_peers >= MAX_PEERS {
                tracing::warn!("At peer limit ({MAX_PEERS}), skipping ({peer})");
                continue;
            }

            active_peers += 1;

            tokio::spawn(async move {
                let mut stream = match timeout(CONNECTION_TIMEOUT, TcpStream::connect(peer)).await {
                    Ok(Ok(stream)) => stream,
                    Ok(Err(err)) => {
                        tracing::error!("Failed to connect to {peer}: {err}");
                        return;
                    },
                    Err(_) => {
                        tracing::error!("Timeout connecting to {peer}");
                        return;
                    },
                };

                tracing::debug!("TCP connected to {peer}");

                match handshake::perform(&mut stream, info_hash, &peer_id).await {
                    Ok(()) => {
                        tracing::debug!("Handshake successful with {peer}");
                    },
                    Err(err) => {
                        tracing::error!("Failed to perform handshake with {peer}: {err}");
                    },
                }
            });
        }
    }

    tokio::signal::ctrl_c().await.ok();

    Ok(())
}
