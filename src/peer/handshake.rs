use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const BITTORRENT_PROTOCOL: &str = "BitTorrent protocol";

pub async fn perform(stream: &mut TcpStream, info_hash: &[u8; 20], peer_id: &[u8; 20]) -> anyhow::Result<()> {
    let mut buf = Vec::with_capacity(68);

    // The handshake starts with character ninteen (decimal) followed by the string 'BitTorrent
    // protocol'.
    buf.push(BITTORRENT_PROTOCOL.len() as u8);
    buf.extend_from_slice(BITTORRENT_PROTOCOL.as_bytes());

    // After the fixed headers come eight reserved bytes, which are all zero in all current
    // implementations.
    buf.extend_from_slice(&[0u8; 8]);

    // Next comes the 20 byte sha1 hash of the bencoded form of the info value from the metainfo file.
    buf.extend_from_slice(info_hash);

    // After the download hash comes
    // the 20-byte peer id which is reported in tracker requests and contained in peer lists in tracker
    // responses.
    buf.extend_from_slice(peer_id);

    stream.write_all(&buf).await?;
    stream.flush().await?;

    let mut response = [0u8; 68];
    stream.read_exact(&mut response).await?;

    if &response[1..20] != BITTORRENT_PROTOCOL.as_bytes() {
        anyhow::bail!("invalid protocol")
    }

    if response[28..48] != *info_hash {
        anyhow::bail!("mismatched info_hash")
    }

    tracing::debug!("handshake successful");

    Ok(())
}
