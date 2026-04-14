use crate::peer::bitfield::Bitfield;

pub struct PeerState {
    am_choked: bool,
    am_interested: bool,
    peer_bitfield: Bitfield,
}

impl PeerState {
    pub fn new(piece_count: usize) -> Self {
        Self { am_choked: true, am_interested: false, peer_bitfield: Bitfield::new(piece_count) }
    }
}
