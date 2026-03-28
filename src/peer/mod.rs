use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

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

        timeout(Duration::from_secs(8), TcpStream::connect(SocketAddrV4::new(self.ip, self.port))).await?
    }

    pub async fn handshake(&self, stream: &mut TcpStream, info_hash: [u8; 20], peer_id: &str) -> io::Result<()> {
        const BITTORRENT_PROTOCOL: &str = "BitTorrent protocol";

        let mut handshake = Vec::with_capacity(68);

        // The handshake starts with character ninteen (decimal) followed by the string 'BitTorrent
        // protocol'.
        handshake.push(19);
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
}
