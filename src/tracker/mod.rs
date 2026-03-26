use std::net::Ipv4Addr;

use bytes::Bytes;
use reqwest::{Client, StatusCode, Url};

use crate::protocol::Bencode;
use crate::protocol::decoder::Decoder;
use crate::tracker::error::TrackerError;

mod error;

pub struct Tracker {
    peer_id: String,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    compact: u8,
    announce: Option<String>,
    info_hash: Bytes,
    http_client: Client,
}

impl Tracker {
    pub fn new(
        peer_id: String,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        compact: u8,
        announce: Option<String>,
        info_hash: Bytes,
    ) -> Self {
        Self {
            peer_id,
            port,
            uploaded,
            downloaded,
            left,
            compact,
            announce,
            info_hash,
            http_client: reqwest::Client::new(),
        }
    }

    fn build_url(&self) -> Result<Url, TrackerError> {
        // reqwest can't parse binary values :(((((
        // https://github.com/seanmonstar/reqwest/issues/1613
        let announce = self.announce.as_ref().ok_or(TrackerError::NoAnnounce)?;
        let info_hash = urlencoding::encode_binary(&self.info_hash);

        let base_url = format!("{announce}?info_hash={info_hash}");

        Url::parse_with_params(
            &base_url,
            &[
                ("peer_id", &self.peer_id),
                ("port", &self.port.to_string()),
                ("uploaded", &self.uploaded.to_string()),
                ("downloaded", &self.downloaded.to_string()),
                ("left", &self.left.to_string()),
                ("compact", &self.compact.to_string()),
            ],
        )
        .map_err(|err| TrackerError::UrlParse(err.to_string()))
    }

    pub async fn get(&self) -> Result<Vec<(Ipv4Addr, u16)>, TrackerError> {
        let url = self.build_url().unwrap();
        let res = self.http_client.get(url).send().await.map_err(TrackerError::RequestError)?;

        if res.status() == StatusCode::OK {
            let mut bytes = {
                let bytes = res.bytes().await.map_err(TrackerError::RequestError)?;
                bytes.into_iter()
            };

            return decode_response(&mut bytes);
        }

        unimplemented!()
    }
}

fn decode_response<T: Iterator<Item = u8>>(bytes: &mut T) -> Result<Vec<(Ipv4Addr, u16)>, TrackerError> {
    let mut decoder = Decoder::new(bytes);
    let bencode = decoder.decode().map_err(TrackerError::Bencode)?;

    let dict = match bencode {
        Bencode::Dictionary(dict) => dict,
        _ => return Err(TrackerError::WrongBencodeType("dict".into())),
    };

    let peers_bencode = dict.get("peers").ok_or(TrackerError::InvalidResponse)?;

    // (IP, PORT)
    let mut result = vec![];
    match peers_bencode {
        Bencode::String(bytes) => {
            for chunks in bytes.chunks_exact(6) {
                let ip = Ipv4Addr::new(chunks[0], chunks[1], chunks[2], chunks[3]);
                let port = u16::from_be_bytes([chunks[4], chunks[5]]);

                result.push((ip, port));
            }
        },
        _ => return Err(TrackerError::WrongBencodeType("string".into())),
    };

    Ok(result)
}
