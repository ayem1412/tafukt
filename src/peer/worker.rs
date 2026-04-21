use std::net::SocketAddr;
use std::time::Duration;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::peer::command::PeerCommand;
use crate::peer::event::{PeerEvent, PeerEventMessage};
use crate::peer::handshake;
use crate::peer::message::Message;

const KEEPALIVE_PERIOD: Duration = Duration::from_secs(120);
pub const BLOCK_SIZE: u16 = 16 * 1024;

pub struct PeerWorker {
    addr: SocketAddr,
    stream: BufWriter<TcpStream>,
    peer_event_tx: mpsc::Sender<PeerEventMessage>,
}

impl PeerWorker {
    pub fn new(addr: SocketAddr, stream: TcpStream, peer_event_tx: mpsc::Sender<PeerEventMessage>) -> Self {
        Self { addr, stream: BufWriter::with_capacity(64 * 1024, stream), peer_event_tx }
    }

    pub async fn run(&mut self, info_hash: [u8; 20], peer_id: [u8; 20]) {
        if let Err(err) = self.try_run(info_hash, peer_id).await {
            tracing::error!("[PeerWorker]: Peer {} disconnected: {err}", self.addr);
        };

        let _ = self.emit(PeerEvent::Disconnected).await;
    }

    async fn try_run(&mut self, info_hash: [u8; 20], peer_id: [u8; 20]) -> anyhow::Result<()> {
        handshake::perform(&mut self.stream, &info_hash, &peer_id).await?;
        tracing::debug!("[PeerWorker]: Peer {} handshake performed successfully", self.addr);

        let mut keepalive_interval = interval(KEEPALIVE_PERIOD);
        keepalive_interval.tick().await;

        self.send(Message::Interested).await?;

        let (peer_cmd_tx, mut peer_cmd_rx) = mpsc::channel(64);
        self.emit(PeerEvent::Connected(peer_cmd_tx)).await?;

        let mut read_buf = BytesMut::with_capacity(32 * 1024);

        loop {
            tokio::select! {
                result = self.stream.read_buf(&mut read_buf) => {
                    let n = result?;
                    if n == 0 {
                        tracing::error!("[PeerWorker]: Peer {} closed connection EOF", self.addr);
                        break;
                    }

                    if let Some(message) = Message::decode(&mut read_buf) {
                        self.handle_message(message).await?;
                    }
                }
                Some(peer_cmd) = peer_cmd_rx.recv() => {
                    self.handle_peer_cmd(peer_cmd).await?;
                    self.stream.flush().await?;
                }
                _ = keepalive_interval.tick() => {
                    tracing::debug!("[PeerWorker]: Sending KeepAlive to Peer {}", self.addr);
                    self.send(Message::KeepAlive).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_message(&self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::KeepAlive => {},
            Message::Choke => self.emit(PeerEvent::Choke).await?,
            Message::Unchoke => self.emit(PeerEvent::Unchoke).await?,
            Message::Interested => todo!(),
            Message::NotInterested => todo!(),
            Message::Have(piece_index) => self.emit(PeerEvent::Have(piece_index)).await?,
            Message::Bitfield(bits) => self.emit(PeerEvent::Bitfield(bits)).await?,
            Message::Request { index: _, begin: _, length: _ } => todo!(),
            Message::Piece { index, begin, data } => {
                self.emit(PeerEvent::Block { piece_index: index, begin, data }).await?
            },
        }

        Ok(())
    }

    async fn handle_peer_cmd(&mut self, peer_cmd: PeerCommand) -> anyhow::Result<()> {
        match peer_cmd {
            PeerCommand::Request { index, begin, length } => {
                tracing::debug!("[PeerWorker]: Requesting Piece {index} (begin {begin} length {length})");

                self.stream.write_all(&Message::Request { index, begin, length }.encode()).await?;
            },
        }

        Ok(())
    }

    /// Emits an event to the [`Engine`].
    async fn emit(&self, event: PeerEvent) -> anyhow::Result<()> {
        self.peer_event_tx.send(PeerEventMessage { addr: self.addr, event }).await?;
        Ok(())
    }

    async fn send(&mut self, message: Message) -> anyhow::Result<()> {
        self.stream.write_all(&message.encode()).await?;
        self.stream.flush().await?;

        Ok(())
    }
}
