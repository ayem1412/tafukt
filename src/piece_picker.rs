use crate::peer::bitfield::Bitfield;

pub struct PiecePicker {
    pub our_bitfield: Bitfield,
    availability: Vec<u32>,
    piece_count: usize,
}

impl PiecePicker {
    pub fn new(our_bitfield: Bitfield) -> Self {
        let piece_count = our_bitfield.piece_count;
        Self { our_bitfield, availability: vec![0; piece_count], piece_count }
    }

    pub fn register_peer_bitfield(&mut self, peer_bitfield: &Bitfield) {
        for index in 0..self.piece_count {
            if peer_bitfield.has_piece(index) {
                self.availability[index] += 1;
            }
        }
    }
}
