use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::disk_manager::{Block, DiskEvent};
use crate::metainfo::info_dictionary::InfoDictionary;
use crate::peer::bitfield::Bitfield;
use crate::peer::command::PeerCommand;
use crate::peer::event::{PeerEvent, PeerEventMessage};
use crate::peer::state::PeerState;
use crate::piece::PieceManager;

const MAX_REQUEST_PIPELINE: usize = 8;
const BLOCK_SIZE: u32 = 16 * 1024;

pub struct Engine {
    info_dictionary: Arc<InfoDictionary>,
    peer_states: HashMap<SocketAddr, PeerState>,
    piece_manager: PieceManager,
    peer_event_rx: mpsc::Receiver<PeerEventMessage>,
    disk_event_rx: mpsc::Receiver<DiskEvent>,
    block_tx: mpsc::Sender<Block>,
}

impl Engine {
    pub fn new(
        info_dictionary: Arc<InfoDictionary>,
        piece_manager: PieceManager,
        peer_event_rx: mpsc::Receiver<PeerEventMessage>,
        disk_event_rx: mpsc::Receiver<DiskEvent>,
        block_tx: mpsc::Sender<Block>,
    ) -> Self {
        Self { info_dictionary, peer_states: HashMap::new(), piece_manager, peer_event_rx, disk_event_rx, block_tx }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(event_message) = self.peer_event_rx.recv() => {
                    self.handle_peer_event_message(event_message).await;
                }
                Some(disk_event) = self.disk_event_rx.recv() => {
                    self.handle_disk_event(disk_event);
                }
            }

            if self.piece_manager.is_complete() {
                break;
            }
        }
    }

    async fn handle_peer_event_message(&mut self, event_message: PeerEventMessage) {
        let addr = event_message.addr;

        match event_message.event {
            PeerEvent::Connected(peer_cmd_tx) => {
                self.peer_states.insert(addr, PeerState::new(peer_cmd_tx));

                tracing::debug!("[Engine]: Peer {addr} connected, ({} peers total)", self.peer_states.len());
            },
            PeerEvent::Choke => {
                if let Some(peer_state) = self.peer_states.get_mut(&addr) {
                    tracing::debug!(
                        "[Engine]: Peer {addr} choked us - recovering {} in-flight blocks",
                        peer_state.in_flight.len()
                    );

                    peer_state.am_choked = true;

                    let stale: Vec<u32> = peer_state.in_flight.drain().collect();
                    for begin in stale {
                        if let Some(current_piece_idx) = peer_state.current_piece_idx {
                            let length = {
                                let piece_len = self.info_dictionary.piece_len(current_piece_idx);
                                (piece_len as u32 - begin).min(BLOCK_SIZE)
                            };

                            peer_state.remaining_blocks.push_front((begin, length));
                        }
                    }
                }
            },
            PeerEvent::Unchoke => {
                if let Some(peer_state) = self.peer_states.get_mut(&addr) {
                    tracing::debug!("[Engine]: Peer {addr} unchoked us - populating request pipeline");

                    peer_state.am_choked = false;

                    self.populate_request_pipeline(addr).await;
                }
            },
            PeerEvent::Have(piece_index) => {
                if let Some(peer_state) = self.peer_states.get_mut(&addr) {
                    tracing::debug!("[Engine]: Peer {addr} has Piece {piece_index} - populating request pipeline");

                    peer_state.bitfield.set(piece_index as usize);

                    if peer_state.current_piece_idx.is_none() {
                        self.populate_request_pipeline(addr).await;
                    }
                }
            },
            PeerEvent::Bitfield(bits) => {
                tracing::debug!("[Engine]: Peer {addr} sent their Bitfield");

                if let Some(peer_state) = self.peer_states.get_mut(&addr) {
                    peer_state.bitfield = Bitfield::from_bytes(bits, self.info_dictionary.piece_count());
                    self.populate_request_pipeline(addr).await;
                }
            },
            PeerEvent::Block { piece_index, begin, data } => {
                tracing::debug!(
                    "[Engine]: Peer {addr} sent Piece {piece_index} (Block begin {begin} data len {})",
                    data.len()
                );

                let _ = self.block_tx.send(Block { index: piece_index, begin, data: data.into() }).await;

                let have = self.piece_manager.have.count_ones();
                let total = self.info_dictionary.piece_count();
                tracing::info!("{have}/{total} ({:.1})%", have as f64 / total as f64 * 100.0);

                if let Some(peer_state) = self.peer_states.get_mut(&addr) {
                    peer_state.in_flight.remove(&begin);

                    if peer_state.in_flight.is_empty() && peer_state.remaining_blocks.is_empty() {
                        peer_state.current_piece_idx = None;
                    }
                }

                self.populate_request_pipeline(addr).await;
            },
            PeerEvent::Disconnected => {
                if let Some(peer_state) = self.peer_states.remove(&addr) {
                    if let Some(idx) = peer_state.current_piece_idx {
                        self.piece_manager.release(idx);

                        tracing::debug!("[Engine]: Peer {addr} disconnected - released Piece {idx}");
                    } else {
                        tracing::debug!("[Engine]: Peer {addr} disconnected");
                    }
                }
            },
        }
    }

    fn handle_disk_event(&mut self, disk_event: DiskEvent) {
        match disk_event {
            DiskEvent::PieceVerified(index) => self.piece_manager.mark_have(index),
            DiskEvent::PieceFailed(index) => self.piece_manager.release(index),
        }
    }

    async fn populate_request_pipeline(&mut self, addr: SocketAddr) {
        let Some(peer_state) = self.peer_states.get_mut(&addr) else {
            return;
        };

        if peer_state.am_choked {
            return;
        }

        if peer_state.current_piece_idx.is_none() && peer_state.remaining_blocks.is_empty() {
            if let Some(piece_idx) = self.piece_manager.claim_piece(&peer_state.bitfield) {
                peer_state.in_flight.clear();
                peer_state.current_piece_idx = Some(piece_idx);
                peer_state.remaining_blocks = populate_remaining_blocks(self.info_dictionary.piece_len(piece_idx) as u32);
            } else {
                return;
            }
        }

        while peer_state.in_flight.len() < MAX_REQUEST_PIPELINE {
            let Some((begin, length)) = peer_state.remaining_blocks.pop_front() else {
                break;
            };
            let Some(index) = peer_state.current_piece_idx else {
                break;
            };

            if peer_state.peer_cmd_tx.send(PeerCommand::Request { index, begin, length }).await.is_err() {
                break;
            }

            peer_state.in_flight.insert(begin);
        }
    }

    fn is_endgame(&self) -> bool {
        self.piece_manager.have.count_zeros() <= 5
    }

    fn endgame_mode(&mut self) {
        for peer_state in self.peer_states.values_mut() {
            peer_state.in_flight.clear();
            peer_state.current_piece_idx = None;
            peer_state.remaining_blocks.clear();
        }
    }
}

fn populate_remaining_blocks(piece_len: u32) -> VecDeque<(u32, u32)> {
    let mut blocks = VecDeque::new();
    let mut offset = 0;

    while offset < piece_len {
        let len = (piece_len - offset).min(BLOCK_SIZE);
        blocks.push_back((offset, len));
        offset += len;
    }

    blocks
}
