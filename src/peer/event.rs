use std::net::SocketAddr;

use tokio::sync::mpsc;

use crate::peer::command::PeerCommand;

pub struct PeerEventMessage {
    pub addr: SocketAddr,
    pub event: PeerEvent,
}

pub enum PeerEvent {
    Connected(mpsc::Sender<PeerCommand>),
    Bitfield(Vec<u8>),
}
