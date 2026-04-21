use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::Level;

use crate::disk_manager::DiskManager;
use crate::engine::Engine;
use crate::metainfo::Metainfo;
use crate::peer::worker::PeerWorker;
use crate::piece::PieceManager;
use crate::protocol::decoder::Decoder;

mod disk_manager;
mod engine;
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
    // let file_content = std::fs::read("./torrents/debian.iso.torrent").unwrap();
    let file_content = include_bytes!("../torrents/debian.iso.torrent");
    let mut bytes = file_content.iter().copied();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode().unwrap();

    let metainfo = Metainfo::try_from(result)?;
    let info_hash = metainfo.info_hash();
    let announce_url = metainfo.announce.expect("Trackless torrents are not supported.");

    let info = metainfo.info;

    let name = info.name.clone();
    let length = info.length();
    tracing::trace!("LENGTH: {length}");
    let piece_length = info.piece_length;
    let piece_count = info.piece_count();

    tracing::debug!("Torrent {name} ({piece_count} pieces x {piece_length} bytes = {length} bytes total)");

    let peer_id = util::generate_peer_id();

    let (peers_tx, mut peers_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        tracker::announce_loop(&announce_url, &info_hash, &peer_id, length, peers_tx).await;
    });

    // let mut valid_peers = Arc::new(RwLock::new(HashSet::new()));
    // let mut active_peers = 0;

    // let piece_manager = Arc::new(Mutex::new(PieceManager::new(piece_count, piece_length, length)));
    /* let disk_manager = Arc::new(tokio::sync::Mutex::new(DiskManager::new(
        Path::new(&name),
        length,
        piece_length,
        Arc::clone(&piece_manager),
        info,
    )?)); */
    // let mut handles = vec![];
    let info = Arc::new(info);
    let mut piece_manager = PieceManager::new(piece_count);

    let (disk_event_tx, disk_event_rx) = mpsc::channel(64);

    let mut disk_manager = DiskManager::new(Path::new(&name), length, Arc::clone(&info), disk_event_tx)?;
    disk_manager.resume(&mut piece_manager).await;

    if piece_manager.is_complete() {
        tracing::info!("Already complete");
        return Ok(());
    }

    let (block_tx, block_rx) = mpsc::channel(512);
    tokio::spawn(disk_manager.run(block_rx));

    let (peer_event_tx, peer_event_rx) = mpsc::channel(1024);
    let mut engine = Engine::new(info, piece_manager, peer_event_rx, disk_event_rx, block_tx);
    tokio::spawn(async move { engine.run().await });

    // let (peer_cmd_tx, peer_cmd_rx) = mpsc::channel(62);
    while let Some(addresses) = peers_rx.recv().await {
        for addr in addresses {
            tracing::debug!("Received peer: {addr}");

            /* let piece_manager = Arc::clone(&piece_manager);
            let disk_manager = Arc::clone(&disk_manager); */

            let peer_event_tx = peer_event_tx.clone();
            // let peer_cmd_tx = peer_cmd_tx.clone();

            tokio::spawn(async move {
                match timeout(CONNECTION_TIMEOUT, TcpStream::connect(addr)).await {
                    Ok(Ok(stream)) => {
                        PeerWorker::new(addr, stream, peer_event_tx).run(info_hash, peer_id).await;
                    },
                    Ok(Err(err)) => {
                        tracing::error!("Failed to connect to {addr}: {err}");
                    },
                    Err(_) => {
                        tracing::error!("Timeout connecting to {addr}");
                    },
                };

                // PeerWorker::new(addr, stream, piece_manager, disk_manager).run(info_hash,
                // peer_id, piece_count).await;
            });

            // handles.push(handle);

            /* if active_peers >= MAX_PEERS {
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
            }); */
        }
    }

    /* for handle in handles {
        let _ = handle.await;
    } */

    Ok(())
}
