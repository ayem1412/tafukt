use bitvec::order::Msb0;
use bitvec::vec::BitVec;

#[derive(Debug, Default)]
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

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_raw_slice()
    }

    pub fn has(&self, index: usize) -> bool {
        self.data.get(index).map(|b| *b).unwrap_or(false)
    }

    pub fn set(&mut self, index: usize) {
        self.data.set(index, true);
    }

    pub fn missing(&self, peer_bitfield: &Bitfield) -> impl Iterator<Item = usize> {
        (0..self.piece_count).filter(|&index| !self.has(index) && peer_bitfield.has(index))
    }

    pub fn is_complete(&self) -> bool {
        self.data.all()
    }

    pub fn count_ones(&self) -> usize {
        self.data.count_ones()
    }

    pub fn count_zeros(&self) -> usize {
        self.data.count_zeros()
    }
}
