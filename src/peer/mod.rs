use std::net::SocketAddr;
use std::sync::Arc;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

use crate::peer::bitfield::Bitfield;
use crate::peer::message::Message;
use crate::piece_picker::PiecePicker;

pub mod bitfield;
pub mod message;
mod piece;

const BITTORRENT_PROTOCOL: &str = "BitTorrent protocol";

pub struct PeerSession {
    address: SocketAddr,
    stream: TcpStream,
    peer_bitfield: Bitfield,
    peer_choking_us: bool,
    we_choking_peer: bool,
    we_interested: bool,
    piece_picker: Arc<Mutex<PiecePicker>>,
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
        }
    }

    pub async fn handshake(&mut self, info_hash: [u8; 20], peer_id: &[u8; 20]) -> anyhow::Result<()> {
        let mut buf = Vec::with_capacity(68);

        // The handshake starts with character ninteen (decimal) followed by the string 'BitTorrent
        // protocol'.
        buf.push(BITTORRENT_PROTOCOL.len() as u8);
        buf.extend_from_slice(BITTORRENT_PROTOCOL.as_bytes());

        // After the fixed headers come eight reserved bytes, which are all zero in all current
        // implementations.
        buf.extend_from_slice(&[0u8; 8]);

        // Next comes the 20 byte sha1 hash of the bencoded form of the info value from the metainfo file.
        buf.extend_from_slice(&info_hash);

        // After the download hash comes
        // the 20-byte peer id which is reported in tracker requests and contained in peer lists in tracker
        // responses.
        buf.extend_from_slice(peer_id);

        self.stream.write_all(&buf).await?;
        self.stream.flush().await?;

        let mut response = [0u8; 68];
        self.stream.read_exact(&mut response).await?;

        if &response[1..20] != BITTORRENT_PROTOCOL.as_bytes() {
            anyhow::bail!("invalid protocol")
        }

        if response[28..48] != info_hash {
            anyhow::bail!("mismatched info_hash")
        }

        tracing::debug!("handshake successful");

        Ok(())
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
            piece_picker.our_bitfield.our_missing_pieces(&self.peer_bitfield).next().is_some()
        };

        if is_useful {
            self.stream.write_all(&Message::Interested.encode()).await?;
            self.we_interested = true;
            tracing::debug!("Sent interested to {}", self.address);
        }

        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut buf = BytesMut::with_capacity(64 * 1024);

        loop {
            tokio::select! {
                result = self.stream.read_buf(&mut buf) => {
                    let n = result?;
                    if n == 0 {
                        tracing::warn!("Peer {} disconnected", self.address);
                        break;
                    }

                    self.process_messages(&mut buf).await?;
                }
            }
        }

        Ok(())
    }
}
