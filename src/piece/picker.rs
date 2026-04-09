use std::collections::{HashMap, HashSet};

use crate::peer::bitfield::Bitfield;
use crate::piece::in_progress::InProgress;

pub struct PiecePicker {
    pub our_bitfield: Bitfield,
    availability: Vec<u32>,
    piece_count: usize,
    in_progress: HashMap<usize, InProgress>,
}

impl PiecePicker {
    pub fn new(our_bitfield: Bitfield) -> Self {
        let piece_count = our_bitfield.piece_count;
        Self { our_bitfield, availability: vec![0; piece_count], piece_count, in_progress: HashMap::new() }
    }

    pub fn register_peer_bitfield(&mut self, peer_bitfield: &Bitfield) {
        for index in 0..self.piece_count {
            if peer_bitfield.has(index) {
                self.availability[index] += 1;
            }
        }
    }

    pub fn cancel_peer_requests(&mut self, piece_indexes: HashSet<usize>) {
        for index in piece_indexes {
            if let Some(entry) = self.in_progress.get_mut(&index) {
                entry.cancel_unfinished();
            }
        }
    }
}
