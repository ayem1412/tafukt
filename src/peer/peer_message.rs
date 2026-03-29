/// 0 - choke
/// 1 - unchoke
/// 2 - interested
/// 3 - not interested
/// 4 - have
/// 5 - bitfield
/// 6 - request
/// 7 - piece
/// 8 - cancel
#[derive(Debug, PartialEq)]
pub enum Message {
    Choke = 0,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
}

impl From<u8> for Message {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have,
            5 => Self::Bitfield,
            6 => Self::Request,
            7 => Self::Piece,
            8 => Self::Cancel,
            _ => panic!("INVALID MESSAGE ID"),
        }
    }
}

pub struct PeerMessage {
    pub id: Message,
    pub payload: Vec<u8>,
}

impl PeerMessage {
    pub fn new(id: Message, payload: Vec<u8>) -> Self {
        Self { id, payload }
    }
}
