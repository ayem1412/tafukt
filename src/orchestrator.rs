use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};
use tokio::time::timeout;

use crate::peer::PeerSession;
use crate::peer::bitfield::Bitfield;
use crate::piece_picker::PiecePicker;
use crate::tracker::Tracker;

pub struct Orchestrator {
    tracker_url: String,
    peer_id: [u8; 20],
    info_hash: [u8; 20],
    piece_count: usize,
    piece_picker: Arc<Mutex<PiecePicker>>,
}

impl Orchestrator {
    /// Creates a new [`Orchestrator`].
    pub fn new(tracker_url: String, peer_id: [u8; 20], info_hash: [u8; 20], piece_count: usize) -> Self {
        let piece_picker = Arc::new(Mutex::new(PiecePicker::new(Bitfield::new(piece_count))));
        Self { tracker_url, peer_id, info_hash, piece_count, piece_picker }
    }

    pub async fn run(self) {
        let (peers_tx, mut peers_rx) = mpsc::channel::<Vec<SocketAddr>>(32);

        let tracker_url = self.tracker_url.clone();
        let tracker = Tracker::new(tracker_url, self.info_hash);
        tokio::spawn(async move { tracker.announce_loop(&self.peer_id, 6881, peers_tx).await });

        let mut peers = HashSet::new();
        loop {
            if let Some(addresses) = peers_rx.recv().await {
                for address in addresses {
                    if peers.insert(address) {
                        self.spawn_peer(address).await;
                    }
                }
            }
        }
    }

    pub async fn spawn_peer(&self, address: SocketAddr) {
        let info_hash = self.info_hash;
        let peer_id = self.peer_id;
        let piece_count = self.piece_count;
        let piece_picker = Arc::clone(&self.piece_picker);

        tokio::spawn(async move {
            match try_peer(address, info_hash, peer_id, piece_count, piece_picker).await {
                Ok(()) => tracing::debug!("Spawned peer: {address}"),
                Err(err) => tracing::error!("An error has occurred while spawning peer: {address} error: {err}"),
            }
        });
    }
}

async fn try_peer(
    address: SocketAddr,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    piece_count: usize,
    piece_picker: Arc<Mutex<PiecePicker>>,
) -> anyhow::Result<()> {
    let stream = timeout(Duration::from_secs(10), TcpStream::connect(address)).await??;
    let mut peer_session = PeerSession::new(address, stream, piece_count, piece_picker);
    peer_session.handshake(info_hash, &peer_id).await?;
    peer_session.run().await
}
