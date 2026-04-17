use std::collections::{HashSet, VecDeque};

use tokio::sync::mpsc;

use crate::{metainfo::info_dictionary::InfoDictionary, peer::{bitfield::Bitfield, command::PeerCommand}, piece::PieceManager};

const MAX_REQUEST_PIPELINE: usize = 8;
const BLOCK_SIZE: u32 = 16 * 1024;

/// Peer's current state
pub struct PeerState {
    /// Peer's [`Bitfield`].
    pub bitfield: Bitfield,

    info: InfoDictionary,

    /// Peer is choking us.
    am_choked: bool,

    /// Current `Piece` index we're downloading from this `Peer`.
    current_piece_idx: Option<u32>,

    /// `Begin` offsets of `Blocks` in-flight.
    in_flight: HashSet<u32>,

    /// Remaining blocks we haven't downloaded yet (begin, length).
    remaining_blocks: VecDeque<(u32, u32)>,

    piece_manager: PieceManager,

    /// Peer's command channel to send a [`PeerCommand`] to the `Engine`.
    peer_cmd_tx: mpsc::Sender<PeerCommand>,
}

impl PeerState {
    /// Creates a new [`PeerState`].
    pub fn new(piece_count: usize, info: InfoDictionary, piece_manager: PieceManager, peer_cmd_tx: mpsc::Sender<PeerCommand>) -> Self {
        Self {
            bitfield: Bitfield::new(piece_count),
            info,
            am_choked: true,
            current_piece_idx: None,
            in_flight: HashSet::new(),
            remaining_blocks: VecDeque::new(),
            piece_manager,
            peer_cmd_tx,
        }
    }

    pub fn populate_request_pipeline(&mut self) {
        if self.am_choked { return; }

        if self.current_piece_idx.is_none() && self.remaining_blocks.is_empty() {
            self.in_flight.clear();

            if let Some(piece_idx) = self.piece_manager.claim_piece(&self.bitfield) {
                self.current_piece_idx = Some(piece_idx);
                self.remaining_blocks = populate_remaining_blocks(self.info.piece_len(piece_idx) as u32);
            } else {
                return;
            }
        }

        while self.in_flight.len() < MAX_REQUEST_PIPELINE {
            let Some((begin, length)) = self.remaining_blocks.pop_front() else { break; };
            let Some(index) = self.current_piece_idx else { break; };

            if self.peer_cmd_tx.try_send(PeerCommand::Request { index, begin, length }).is_err() { break; }

            self.in_flight.insert(begin);
        }
    }
}

fn populate_remaining_blocks(piece_length: u32) -> VecDeque<(u32, u32)> {
    let mut blocks = VecDeque::new();
    let mut offset = 0;

    while offset < piece_length {
        let len = (piece_length - offset).min(BLOCK_SIZE);
        blocks.push_back((offset, len));
        offset += len;
    }

    blocks
}
