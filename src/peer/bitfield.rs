use bitvec::order::Msb0;
use bitvec::vec::BitVec;

#[derive(Debug)]
pub struct Bitfield {
    data: BitVec<u8, Msb0>,
    pub piece_count: usize,
}

impl Bitfield {
    pub fn new(piece_count: usize) -> Self {
        Self { data: BitVec::repeat(false, piece_count), piece_count }
    }

    pub fn from_bytes(data: Vec<u8>, piece_count: usize) -> Self {
        Self { data: BitVec::from_vec(data), piece_count }
    }

    pub fn has_piece(&self, index: usize) -> bool {
        self.data.get(index).map(|b| *b).unwrap_or(false)
    }

    pub fn set_piece(&mut self, index: usize) {
        self.data.set(index, true);
    }

    pub fn our_missing_pieces(&self, peer_bitfield: &Bitfield) -> impl Iterator<Item = usize> {
        (0..self.piece_count).filter(|&index| !self.has_piece(index) && peer_bitfield.has_piece(index))
    }
}
