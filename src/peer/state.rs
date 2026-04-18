use std::collections::{HashSet, VecDeque};

use tokio::sync::mpsc;

use crate::peer::bitfield::Bitfield;
use crate::peer::command::PeerCommand;

const MAX_REQUEST_PIPELINE: usize = 8;
const BLOCK_SIZE: u32 = 16 * 1024;

/// Peer's current state
pub struct PeerState {
    /// Peer's [`Bitfield`].
    pub bitfield: Bitfield,

    /// Peer is choking us.
    pub am_choked: bool,

    /// Current `Piece` index we're downloading from this `Peer`.
    pub current_piece_idx: Option<u32>,

    /// `Begin` offsets of `Blocks` in-flight.
    pub in_flight: HashSet<u32>,

    /// Remaining blocks we haven't downloaded yet (begin, length).
    pub remaining_blocks: VecDeque<(u32, u32)>,

    /// Peer's command channel to send a [`PeerCommand`] to the `Engine`.
    pub peer_cmd_tx: mpsc::Sender<PeerCommand>,
}

impl PeerState {
    /// Creates a new [`PeerState`].
    pub fn new(peer_cmd_tx: mpsc::Sender<PeerCommand>) -> Self {
        Self {
            bitfield: Bitfield::default(),
            am_choked: true,
            current_piece_idx: None,
            in_flight: HashSet::new(),
            remaining_blocks: VecDeque::new(),
            peer_cmd_tx,
        }
    }
}
