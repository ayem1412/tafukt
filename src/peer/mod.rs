use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

use crate::peer::peer_message::{Message, PeerMessage};
use crate::peer::piece::Piece;

mod bitfield;
pub mod peer_message;
mod piece;

#[derive(Debug)]
pub struct Peer {
    ip: Ipv4Addr,
    port: u16,
}

impl Peer {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub async fn connect(&self) -> io::Result<TcpStream> {
        println!("CONNECTING TO: {}", self.addr());

        timeout(Duration::from_secs(4), TcpStream::connect(SocketAddrV4::new(self.ip, self.port))).await?
    }

    pub async fn handshake(&self, stream: &mut TcpStream, info_hash: [u8; 20], peer_id: &str) -> io::Result<()> {
        const BITTORRENT_PROTOCOL: &str = "BitTorrent protocol";

        let mut handshake = Vec::with_capacity(68);

        // The handshake starts with character ninteen (decimal) followed by the string 'BitTorrent
        // protocol'.
        handshake.push(BITTORRENT_PROTOCOL.len() as u8);
        handshake.extend_from_slice(BITTORRENT_PROTOCOL.as_bytes());

        // After the fixed headers come eight reserved bytes, which are all zero in all current
        // implementations.
        handshake.extend_from_slice(&[0u8; 8]);

        // Next comes the 20 byte sha1 hash of the bencoded form of the info value from the metainfo file.
        handshake.extend_from_slice(&info_hash);

        // After the download hash comes
        // the 20-byte peer id which is reported in tracker requests and contained in peer lists in tracker
        // responses.
        handshake.extend_from_slice(peer_id.as_bytes());

        println!("WRITING!");
        stream.write_all(&handshake).await?;
        stream.flush().await?;

        println!("READING!");
        let mut response = [0u8; 68];
        timeout(Duration::from_secs(10), stream.read_exact(&mut response)).await??;

        if &response[1..20] != BITTORRENT_PROTOCOL.as_bytes() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid protocol"));
        }

        if &response[28..48] != info_hash.as_ref() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "mismatched info_hash"));
        }

        println!("SUCCESSFUL HANDSHAKE: {}", self.addr());

        Ok(())
    }

    pub async fn read_message(&self, stream: &mut TcpStream) -> io::Result<Option<PeerMessage>> {
        let mut length = [0u8; 4];
        stream.read_exact(&mut length).await?;
        let length = u32::from_be_bytes(length);

        if length == 0 {
            println!("RECEIVED KEEPALIVE");
            return Ok(None);
        }

        let mut message = vec![0u8; length as usize];
        stream.read_exact(&mut message).await?;

        let id = message[0];
        let payload = message[1..].to_vec();

        println!("RECEIVED MESSAGE ID {id} ({} BYTES PAYLOAD)", payload.len());

        Ok(Some(PeerMessage::new(Message::from(id), payload)))
    }

    pub async fn bitfield(&self, stream: &mut TcpStream, piece_count: usize) -> io::Result<()> {
        // amount of bytes needed for the bitfield.
        let bitfield_len = piece_count.div_ceil(8); // `div_ceil` to round up.

        // bitfield message.
        let bitfield = vec![0u8; bitfield_len]; // all 0s because no pieces yet.

        // length of the message.
        let length = 1 + bitfield_len; // 1 for the message ID + the length amount of bytes.

        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&[Message::Bitfield as u8]).await?;
        stream.write_all(&bitfield).await?;
        stream.flush().await?;

        println!("SENT BITFIELD");

        Ok(())
    }

    pub async fn interested(&self, stream: &mut TcpStream) -> io::Result<()> {
        stream.write_all(&[0, 0, 0, 1, Message::Interested as u8]).await?; // 1 for the message length, 2 for the message ID.
        stream.flush().await?;

        println!("SENT INTERESTED");

        Ok(())
    }

    pub async fn unchoke(&self, stream: &mut TcpStream) -> io::Result<()> {
        stream.write_all(&[0, 0, 0, 1, Message::Unchoke as u8]).await?; // 1 for the message length, 1 for the message ID.
        stream.flush().await?;

        println!("SENT UNCHOKE");

        Ok(())
    }

    pub async fn keepalive(&self, stream: &mut TcpStream) -> io::Result<()> {
        stream.write_all(&[0u8; 4]).await?;
        stream.flush().await?;

        Ok(())
    }

    pub async fn request(&self, stream: &mut TcpStream, index: u32, begin: u32, length: u32) -> io::Result<()> {
        let message_length = 13u32;

        stream.write_all(&message_length.to_be_bytes()).await?;
        stream.write_all(&[Message::Request as u8]).await?;
        stream.write_all(&index.to_be_bytes()).await?;
        stream.write_all(&begin.to_be_bytes()).await?;
        stream.write_all(&length.to_be_bytes()).await?;

        stream.flush().await?;

        Ok(())
    }

    pub async fn piece(&self, payload: Vec<u8>) -> io::Result<Piece> {
        let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);

        let block = &payload[8..];

        Ok(Piece::new(index, begin, block.into()))
    }
}
