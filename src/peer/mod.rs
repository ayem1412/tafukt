use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::interval;

use crate::disk_manager::{Block, DiskManager};
use crate::peer::bitfield::Bitfield;
use crate::peer::message::Message;
use crate::piece::PieceManager;

pub mod bitfield;
pub mod handshake;
pub mod message;
mod piece;
mod swarm;

const KEEPALIVE_SECONDS: u64 = 120;
const PIPELINE_DEPTH: usize = 8;
const BLOCK_SIZE: u16 = 16 * 1024;

pub struct PeerWorker {
    addr: SocketAddr,
    stream: TcpStream,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    am_choked: bool,
    piece_count: usize,
    piece_length: u64,
    length: u64,
    peer_bitfield: Bitfield,
    remaining_blocks: VecDeque<(u32, u32)>,
    current_piece_idx: Option<u32>,
    piece_manager: Arc<Mutex<PieceManager>>,
    in_flight: HashMap<u32, u32>,
    disk_manager: Arc<Mutex<DiskManager>>,
}

impl PeerWorker {
    pub fn new(
        addr: SocketAddr,
        stream: TcpStream,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        piece_count: usize,
        piece_length: u64,
        length: u64,
        piece_manager: Arc<Mutex<PieceManager>>,
        disk_manager: Arc<Mutex<DiskManager>>,
    ) -> Self {
        Self {
            addr,
            stream,
            info_hash,
            peer_id,
            am_choked: true,
            piece_count,
            piece_length,
            length,
            peer_bitfield: Bitfield::new(piece_count),
            remaining_blocks: VecDeque::new(),
            current_piece_idx: None,
            piece_manager,
            in_flight: HashMap::new(),
            disk_manager,
        }
    }

    pub async fn run(&mut self) {
        if let Err(err) = self.try_run().await {
            tracing::error!("PeerWorker: [Peer {}] disconnected: {err}", self.addr);
        };

        if let Some(idx) = self.current_piece_idx {
            self.piece_manager.lock().unwrap().release(idx);
        }
    }

    async fn try_run(&mut self) -> anyhow::Result<()> {
        handshake::perform(&mut self.stream, &self.info_hash, &self.peer_id).await?;
        tracing::debug!("PeerWorker: [Peer {}] handshake ok", self.addr);

        tracing::debug!("PeerWorker: Interested in [Peer {}]", self.addr);
        self.stream.write_all(&Message::Interested.encode()).await?;

        let mut buf = BytesMut::with_capacity(32 * 1024);
        let mut keepalive_interval = interval(Duration::from_secs(KEEPALIVE_SECONDS));
        keepalive_interval.tick().await;

        loop {
            let can_request =
                !self.am_choked && !self.remaining_blocks.is_empty() && self.in_flight.len() < PIPELINE_DEPTH;

            tokio::select! {
                result = self.stream.read_buf(&mut buf) => {
                    let n = result?;
                    if n == 0 {
                        tracing::error!("PeerWorker: [Peer {}] Closed connection (EOF) - releasing piece {:?}", self.addr, self.current_piece_idx);
                        break;
                    }

                    if let Some(msg) = Message::decode(&mut buf) {
                        self.handle_message(msg).await?;
                    }
                }
                _ = async {}, if can_request => {
                    self.send_next_request().await?;
                }
                _ = keepalive_interval.tick() => {
                        tracing::debug!("PeerWorker: Sending a KeepAlive message to [Peer {}]", self.addr);
                        self.stream.write_all(&Message::KeepAlive.encode()).await?;
                }
            }

            self.maybe_claim_piece();

            if self.piece_manager.lock().unwrap().is_complete() {
                break;
            }
        }

        Ok(())
    }

    fn piece_len(&self, index: u32) -> u32 {
        let start = index as u64 * self.piece_length;
        let remaining = self.length.saturating_sub(start);

        remaining.min(self.piece_length) as u32
    }

    async fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg {
            Message::KeepAlive => tracing::debug!("PeerWorker: [Peer {}] KeepAlive", self.addr),
            Message::Choke => {
                tracing::debug!("PeerWorker: [Peer {}] Choked us", self.addr);
                self.am_choked = true;
            },
            Message::Unchoke => {
                tracing::debug!("PeerWorker: [Peer {}] Unchoked us", self.addr);
                self.am_choked = false;

                /* if self.peer_bitfield.as_bytes().iter().all(|&i| i == 0) {
                    for i in 0..self.piece_count {
                        self.peer_bitfield.set(i);
                    }
                } */

                self.maybe_claim_piece();
            },
            Message::Interested => todo!(),
            Message::NotInterested => todo!(),
            Message::Have(index) => {
                tracing::debug!("PeerWorker: [Peer {}] Sent us a `Have` message: {index}", self.addr);
            },
            Message::Bitfield(bits) => {
                tracing::debug!("PeerWorker: [Peer {}] Sent us their Bitfield", self.addr);
                self.peer_bitfield = Bitfield::from_bytes(bits, self.piece_count);

                self.maybe_claim_piece();
            },
            Message::Request { index, begin, length } => todo!(),
            Message::Piece { index, begin, data } => {
                if Some(index) != self.current_piece_idx {
                    return Ok(());
                }

                tracing::debug!(
                    "PeerWorker: [Peer {}] Sent us a `Piece` message: index {index} begin {begin}",
                    self.addr
                );

                self.in_flight.remove(&begin);
                self.disk_manager.lock().unwrap().handle_block(Block { index, begin, data: data.to_vec() });

                if self.remaining_blocks.is_empty() && self.in_flight.is_empty() {
                    self.current_piece_idx = None;
                }
            },
        }

        Ok(())
    }

    fn maybe_claim_piece(&mut self) {
        if self.current_piece_idx.is_some() {
            return;
        }

        let mut piece_manager = self.piece_manager.lock().unwrap();
        if let Some(index) = piece_manager.claim_piece(&self.peer_bitfield) {
            let piece_len = self.piece_len(index);

            let blocks = {
                let mut offset = 0u32;
                let mut v = VecDeque::new();

                while offset < piece_len {
                    let len = (piece_len - offset).min(BLOCK_SIZE as u32);
                    v.push_back((offset, len));
                    offset += len;
                }
                v
            };

            tracing::debug!("PeerWorker: [Peer {}] claimed piece: {index} ({} blocks)", self.addr, blocks.len());

            self.current_piece_idx = Some(index);
            self.remaining_blocks = blocks;
            self.in_flight.clear();
        }
    }

    async fn send_next_request(&mut self) -> anyhow::Result<()> {
        if let Some((begin, length)) = self.remaining_blocks.pop_front() {
            let index = self.current_piece_idx.expect("`current_piece_idx` is None");

            tracing::debug!(
                "PeerWorker: Sending `Request` to [Peer {}]: piece {index} (begin {begin} length {length})",
                self.addr
            );
            self.stream.write_all(&Message::Request { index, begin, length }.encode()).await?;
            self.in_flight.insert(begin, length);
        }

        Ok(())
    }
}
