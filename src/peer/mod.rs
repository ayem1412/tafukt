use std::collections::{HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};

use crate::peer::bitfield::Bitfield;
use crate::peer::message::Message;
use crate::piece::picker::PiecePicker;

pub mod bitfield;
pub mod handshake;
pub mod message;
mod piece;
mod swarm;

const KEEPALIVE_SECONDS: u64 = 30;
const PIPELINE_DEPTH: usize = 8;

struct PendingRequest {
    index: usize,
    offset: u32,
    length: u32,
}

pub struct PeerSession {
    address: SocketAddr,
    stream: TcpStream,
    peer_bitfield: Bitfield,
    peer_choking_us: bool,
    we_choking_peer: bool,
    we_interested: bool,
    piece_picker: Arc<Mutex<PiecePicker>>,
    request_pipeline: VecDeque<PendingRequest>,
}

impl PeerSession {
    pub fn new(
        address: SocketAddr,
        stream: TcpStream,
        piece_count: usize,
        piece_picker: Arc<Mutex<PiecePicker>>,
    ) -> Self {
        Self {
            address,
            stream,
            peer_bitfield: Bitfield::new(piece_count),
            peer_choking_us: true,
            we_choking_peer: true,
            we_interested: false,
            piece_picker,
            request_pipeline: VecDeque::new(),
        }
    }

    async fn process_messages(&mut self, buf: &mut BytesMut) -> anyhow::Result<()> {
        while let Some(message) = Message::decode(buf) {
            self.handle_message(message).await?;
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Choke => {
                tracing::debug!("Peer choked us: {}", self.address);
                self.peer_choking_us = true;
            },
            Message::Unchoke => {
                tracing::debug!("Peer unchoked us: {}", self.address);
                self.peer_choking_us = false;
            },
            Message::Bitfield(data) => {
                self.peer_bitfield = Bitfield::from_bytes(data, self.peer_bitfield.piece_count);
                tracing::debug!("Received bitfield: {:?}", self.peer_bitfield);
                self.piece_picker.lock().await.register_peer_bitfield(&self.peer_bitfield);
                self.send_interested_if_useful().await?;
            },
            Message::Request { index, begin, length } => todo!(),
            Message::Piece { index, begin, data } => todo!(),
            Message::KeepAlive | Message::Interested | Message::NotInterested => {},
        }

        Ok(())
    }

    async fn send_interested_if_useful(&mut self) -> anyhow::Result<()> {
        if self.we_interested {
            return Ok(());
        }
        let is_useful = {
            let piece_picker = self.piece_picker.lock().await;
            piece_picker.our_bitfield.missing(&self.peer_bitfield).next().is_some()
        };

        if is_useful {
            self.stream.write_all(&Message::Interested.encode()).await?;
            self.we_interested = true;
            tracing::debug!("Sent interested to {}", self.address);
        }

        Ok(())
    }

    async fn cancel_all_requests(&mut self) {
        let piece_indexes: HashSet<usize> = self.request_pipeline.iter().map(|request| request.index).collect();
        self.request_pipeline.clear();

        if !piece_indexes.is_empty() {
            self.piece_picker.lock().await.cancel_peer_requests(piece_indexes);
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("Session started with {}", self.address);

        {
            let piece_picker = self.piece_picker.lock().await;
            let our_bitfield = piece_picker.our_bitfield.as_bytes().to_vec();
            self.stream.write_all(&Message::Bitfield(our_bitfield).encode()).await?;
        }

        let mut buf = BytesMut::with_capacity(64 * 1024);
        let mut keepalive = interval(Duration::from_secs(KEEPALIVE_SECONDS));

        loop {
            tokio::select! {
                result = self.stream.read_buf(&mut buf) => {
                    let n = result?;
                    if n == 0 {
                        tracing::warn!("Peer {} disconnected", self.address);
                        self.cancel_all_requests().await;
                        break;
                    }
                    self.process_messages(&mut buf).await?;
                }
                _ = keepalive.tick() => {
                    self.stream.write_all(&Message::KeepAlive.encode()).await?;
                }
            }
        }

        Ok(())
    }
}
