use std::net::SocketAddr;

use bytes::Bytes;
use tokio::sync::mpsc;

use crate::peer::command::PeerCommand;

pub struct PeerEventMessage {
    pub addr: SocketAddr,
    pub event: PeerEvent,
}

pub enum PeerEvent {
    Connected(mpsc::Sender<PeerCommand>),
    Choke,
    Unchoke,
    Have(u32),
    Bitfield(Vec<u8>),
    Block { piece_index: u32, begin: u32, data: Bytes },
    Disconnected,
}
