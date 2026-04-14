use std::collections::HashSet;

use crate::peer::bitfield::Bitfield;

mod in_progress;
pub mod picker;

pub struct PieceManager {
    have: Bitfield,
    pending: HashSet<u32>,
    piece_length: u64,
    length: u64,
}

impl PieceManager {
    pub fn new(piece_count: usize, piece_length: u64, length: u64) -> Self {
        Self { have: Bitfield::new(piece_count), pending: HashSet::new(), piece_length, length }
    }

    pub fn release(&mut self, index: u32) {
        if !self.have.has(index as usize) {
            self.pending.remove(&index);
        }
    }

    pub fn claim_piece(&mut self, peer_bitfield: &Bitfield) -> Option<u32> {
        let index = self.have.missing(peer_bitfield).find(|&i| !self.pending.contains(&(i as u32)))?;

        self.pending.insert(index as u32);
        Some(index as u32)
    }

    pub fn is_complete(&self) -> bool {
        self.have.is_complete()
    }

    pub fn piece_len(&self, index: u32) -> u32 {
        let start = index as u64 * self.piece_length;
        let remaining = self.length.saturating_sub(start);

        remaining.min(self.piece_length) as u32
    }
}
