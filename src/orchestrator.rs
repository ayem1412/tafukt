use std::{net::SocketAddr, time::Duration};

use tokio::{net::TcpStream, time::timeout};

use crate::peer::PeerSession;

pub struct Orchestrator {
    peer_id: [u8; 20],
    info_hash: [u8; 20],
}

impl Orchestrator {
    pub fn new(peer_id: [u8; 20], info_hash: [u8; 20]) -> Self {
        Self { peer_id, info_hash }
    }

    pub async fn spawn_peer(self, address: SocketAddr) {
        tokio::spawn(async move {
            match self.try_peer(address).await {
                Ok(()) => tracing::debug!("Spawned peer: {address}"),
                Err(err) => tracing::error!("An error has occurred while spawning peer: {address} error: {err}"),
            }
        });
    }

    async fn try_peer(&self, address: SocketAddr) -> anyhow::Result<()> {
        let stream = timeout(Duration::from_secs(10), TcpStream::connect(address)).await??;
        let mut peer_session = PeerSession::new(address, stream);
        peer_session.handshake(self.info_hash, &self.peer_id).await?;
        peer_session.run().await
    }
}
