use std::collections::HashSet;

use crate::peer::bitfield::Bitfield;

mod in_progress;
pub mod picker;

pub struct PieceManager {
    have: Bitfield,
    pending: HashSet<u32>,
}

impl PieceManager {
    pub fn new(piece_count: usize) -> Self {
        Self { have: Bitfield::new(piece_count), pending: HashSet::new() }
    }

    pub fn claim_piece(&mut self, peer_bitfield: &Bitfield) -> Option<u32> {
        let index = self.have.missing(peer_bitfield).find(|&i| !self.pending.contains(&(i as u32)))?;

        self.pending.insert(index as u32);
        Some(index as u32)
    }
}
