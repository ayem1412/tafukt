use std::{collections::HashMap, net::SocketAddr};

use tokio::sync::mpsc;

use crate::{metainfo::info_dictionary::InfoDictionary, peer::{bitfield::Bitfield, event::{PeerEvent, PeerEventMessage}, state::PeerState}, piece::PieceManager};

pub struct Engine {
    piece_count: usize,
    peer_states: HashMap<SocketAddr, PeerState>,
    peer_rx: mpsc::Receiver<PeerEventMessage>,
}

impl Engine {
    pub fn new(piece_count: usize, peer_rx: mpsc::Receiver<PeerEventMessage>) -> Self {
        Self { piece_count, peer_states: HashMap::new(), peer_rx }
    }

    pub async fn run(&mut self, info: InfoDictionary, piece_manager: PieceManager) {
        tokio::select! {
            Some(event_message) = self.peer_rx.recv() => {
                self.handle_peer_event_message(event_message, info, piece_manager);
            }
        }
    }

    fn handle_peer_event_message(&mut self, event_message: PeerEventMessage, info: InfoDictionary, piece_manager: PieceManager) {
        let addr = event_message.addr;

        match event_message.event {
            PeerEvent::Connected(peer_cmd_tx) => {
                tracing::debug!("[Engine]: Peer {addr} connected, ({} peers total)", self.peer_states.len());
                self.peer_states.insert(addr, PeerState::new(self.piece_count, info, piece_manager, peer_cmd_tx));
            },
            PeerEvent::Bitfield(bits) => {
                tracing::debug!("[Engine]: Peer {addr} sent their Bitfield");
                if let Some(peer_state) = self.peer_states.get_mut(&addr) {
                    peer_state.bitfield = Bitfield::from_bytes(bits, self.piece_count);
                    peer_state.populate_request_pipeline();
                }
            },
        }
    }
}
