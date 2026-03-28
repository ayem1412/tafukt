use std::net::Ipv4Addr;
use std::vec;

use bytes::Bytes;
use reqwest::{Client, StatusCode, Url};

use crate::protocol::Bencode;
use crate::protocol::decoder::Decoder;
use crate::tracker::error::TrackerError;
use crate::tracker::response::TrackerSuccessResponse;

mod error;
mod response;

pub struct Tracker<'a> {
    announce: Option<String>,
    info_hash: &'a Bytes,
    http_client: Client,
}

impl<'a> Tracker<'a> {
    pub fn new(announce: Option<String>, info_hash: &'a Bytes) -> Self {
        Self { announce, info_hash, http_client: reqwest::Client::new() }
    }

    fn build_url(
        &self,
        peer_id: String,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        compact: u8,
    ) -> Result<Url, TrackerError> {
        // reqwest can't parse binary values :(((((
        // https://github.com/seanmonstar/reqwest/issues/1613
        let announce = self.announce.as_ref().ok_or(TrackerError::NoAnnounce)?;
        let info_hash = urlencoding::encode_binary(self.info_hash);

        let base_url = format!("{announce}?info_hash={info_hash}");

        Url::parse_with_params(
            &base_url,
            &[
                ("peer_id", peer_id),
                ("port", port.to_string()),
                ("uploaded", uploaded.to_string()),
                ("downloaded", downloaded.to_string()),
                ("left", left.to_string()),
                ("compact", compact.to_string()),
            ],
        )
        .map_err(|err| TrackerError::UrlParse(err.to_string()))
    }

    pub async fn get_peers(
        &self,
        peer_id: String,
        port: u16,
        uploaded: u64,
        downloaded: u64,
        left: u64,
        compact: u8,
    ) -> Result<TrackerSuccessResponse, TrackerError> {
        let url = self.build_url(peer_id, port, uploaded, downloaded, left, compact)?;
        let res = self.http_client.get(url).send().await.map_err(TrackerError::RequestError)?;
        let bytes = res.bytes().await.map_err(TrackerError::RequestError)?;
        decode_response(&bytes)
    }
}

fn decode_response(bytes: &[u8]) -> Result<TrackerSuccessResponse, TrackerError> {
    let mut iter = bytes.iter().copied();
    let mut decoder = Decoder::new(&mut iter);

    let bencode = decoder.decode().map_err(TrackerError::Bencode)?;

    let dict = match bencode {
        Bencode::Dictionary(dict) => dict,
        _ => return Err(TrackerError::WrongBencodeType("dict".into())),
    };

    if let Some(Bencode::String(reason)) = dict.get("failure reason") {
        return Err(TrackerError::ResponseFailure(String::from_utf8_lossy(reason).into()));
    }

    let interval = match dict.get("interval") {
        Some(Bencode::Integer(value)) => *value as u64,
        Some(_) => return Err(TrackerError::WrongBencodeType("integer".into())),
        None => return Err(TrackerError::ResponseKeyMissing("interval".into())),
    };

    // (IP, PORT)
    let peers = match dict.get("peers") {
        Some(Bencode::String(bytes)) => bytes
            .chunks_exact(6)
            .map(|chunks| {
                let ip = Ipv4Addr::new(chunks[0], chunks[1], chunks[2], chunks[3]);
                let port = u16::from_be_bytes([chunks[4], chunks[5]]);

                (ip, port)
            })
            .collect(),
        Some(_) => return Err(TrackerError::WrongBencodeType("string".into())),
        None => return Err(TrackerError::ResponseKeyMissing("peers".into())),
    };

    Ok(TrackerSuccessResponse::new(interval, peers))
}
