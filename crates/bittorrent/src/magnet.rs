use std::{net::SocketAddr, str::FromStr};

use crate::{hex, util};

#[derive(Debug, thiserror::Error)]
pub enum MagnetError {
    #[error("expected the magnet to start with 'magnet:?'")]
    InvalidMagnetPrefix,
    #[error("not BitTorrent infohash")]
    NotBittorrentHash,
    #[error("the magnet has no 'xt' parameter")]
    MissingXt,
    #[error("invalid infohash base32")]
    InvalidBase32,
    #[error("the infohash must be 40 hex characters or 32 base32 characters")]
    InvalidHashLength,
    #[error(transparent)]
    Hex(#[from] hex::HexError),
}

#[derive(Debug)]
pub struct Magnet {
    /// the info-hash hex encoded, for a total of 40 characters.
    pub info_hash: [u8; 20],
    /// the display name that may be used by the client to display while waiting for metadata.
    pub display_name: Vec<u8>,
    /// tracker url(s).
    pub trackers: Vec<Vec<u8>>,
    /// peer addresse(s).
    pub peer_addresses: Vec<SocketAddr>,
}

// TODO: handle base32
fn parse_xt(xt: &str) -> Result<[u8; 20], MagnetError> {
    let hash = xt
        .strip_prefix("urn:btih:")
        .ok_or(MagnetError::NotBittorrentHash)?;

    match hash.len() {
        40 => Ok(hex::decode_hex_array(hash)?),
        32 => {
            let bytes = data_encoding::BASE32_NOPAD
                .decode(hash.to_uppercase().as_bytes())
                .map_err(|_| MagnetError::InvalidBase32)?;

            bytes.try_into().map_err(|_| MagnetError::InvalidHashLength)
        }
        _ => Err(MagnetError::InvalidHashLength),
    }
}

fn parse_x_pe(xpe: &str) -> Option<SocketAddr> {
    let decoded = util::percent_decode(xpe);
    String::from_utf8(decoded).ok()?.parse().ok()
}

impl FromStr for Magnet {
    type Err = MagnetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let params = s
            .strip_prefix("magnet:?")
            .ok_or(MagnetError::InvalidMagnetPrefix)?
            .split('&')
            .filter_map(|param| param.split_once('='));

        let mut info_hash = None;
        let mut display_name = vec![];
        let mut trackers = vec![];
        let mut peer_addresses = vec![];

        for (key, value) in params {
            match key {
                "xt" => info_hash = Some(parse_xt(value)?),
                "dn" => display_name = util::percent_decode(value),
                "tr" => trackers.push(util::percent_decode(value)),
                "x.pe" => peer_addresses.extend(parse_x_pe(value)),
                _ => {}
            }
        }

        Ok(Self {
            info_hash: info_hash.ok_or(MagnetError::MissingXt)?,
            display_name,
            trackers,
            peer_addresses,
        })
    }
}
