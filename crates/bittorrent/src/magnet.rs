//! Parsing magnet links.
//!
//! A magnet link identifies a torrent without carrying its metadata. It gives
//! you the infohash, and optionally a name, some trackers, and peer hints —
//! enough to start looking for peers, who can then supply the metadata itself.
//!
//! Parsing is done through [`FromStr`](std::str::FromStr), so use `.parse()`:
//!
//! ```
//! use bittorrent::magnet::Magnet;
//!
//! let link = "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=example";
//! let magnet: Magnet = link.parse()?;
//!
//! assert_eq!(magnet.display_name, b"example");
//! # Ok::<(), bittorrent::magnet::MagnetError>(())
//! ```
//!
//! # Tolerance
//!
//! Magnet links are an open format, so unrecognised parameters are ignored
//! rather than rejected. Malformed peer hints are skipped for the same
//! reason — they are optional shortcuts, and losing one costs nothing. Only
//! a missing or unusable infohash fails the parse, since without it there is
//! no torrent to identify.

use std::{net::SocketAddr, str::FromStr};

use crate::{hex, util};

/// A magnet link could not be parsed.
#[derive(Debug, thiserror::Error)]
pub enum MagnetError {
    /// The string did not begin with `magnet:?`.
    #[error("expected the magnet to start with 'magnet:?'")]
    InvalidMagnetPrefix,

    /// The `xt` parameter identified something other than a `BitTorrent`
    /// torrent — magnet links are used for other content types too.
    #[error("not a BitTorrent infohash")]
    NotBittorrentHash,

    /// No `xt` parameter was present, so there is no torrent to identify.
    #[error("the magnet has no 'xt' parameter")]
    MissingXt,

    /// The infohash was 32 characters but not valid base32.
    #[error("invalid base32 infohash")]
    InvalidBase32,

    /// The infohash was neither 40 nor 32 characters long.
    #[error("the infohash must be 40 hex characters or 32 base32 characters")]
    InvalidHashLength,

    /// The infohash was 40 characters but contained a non-hex character.
    #[error(transparent)]
    Hex(#[from] hex::HexError),
}

/// A parsed magnet link.
///
/// Only [`info_hash`](Self::info_hash) is guaranteed to be meaningful. The
/// other fields are hints the link may or may not have carried, and are
/// empty when absent.
#[derive(Debug)]
pub struct Magnet {
    /// The torrent's 20-byte identity, decoded from the link's text form.
    pub info_hash: [u8; 20],

    /// Suggested name, for display while metadata is still being fetched.
    ///
    /// Kept as raw bytes because torrent names have no guaranteed encoding.
    /// Empty if the link had no `dn` parameter.
    pub display_name: Vec<u8>,

    /// Tracker URLs, in the order they appeared.
    ///
    /// May be empty, in which case peers must be found through the DHT.
    pub trackers: Vec<Vec<u8>>,

    /// Peer addresses the link suggested trying directly.
    ///
    /// A shortcut past tracker and DHT lookups. Entries that could not be
    /// parsed as socket addresses are skipped rather than reported.
    pub peer_addresses: Vec<SocketAddr>,
}

/// Decode the `xt` parameter into an infohash.
///
/// The same 20 bytes can be written two ways, and the length says which:
/// 40 characters is hex, 32 is base32.
fn parse_xt(xt: &str) -> Result<[u8; 20], MagnetError> {
    let hash = xt
        .strip_prefix("urn:btih:")
        .ok_or(MagnetError::NotBittorrentHash)?;

    match hash.len() {
        40 => Ok(hex::decode_hex_array(hash)?),

        32 => {
            // The standard alphabet is uppercase, but links in the wild
            // sometimes use lowercase.
            let bytes = data_encoding::BASE32_NOPAD
                .decode(hash.to_uppercase().as_bytes())
                .map_err(|_| MagnetError::InvalidBase32)?;

            // 32 base32 characters always yield exactly 20 bytes.
            bytes.try_into().map_err(|_| MagnetError::InvalidHashLength)
        }

        _ => Err(MagnetError::InvalidHashLength),
    }
}

/// Decode an `x.pe` peer hint, or `None` if it is unusable.
///
/// Handles both `1.2.3.4:6881` and the bracketed IPv6 form
/// `[2001:db8::1]:6881`.
fn parse_x_pe(xpe: &str) -> Option<SocketAddr> {
    let decoded = util::percent_decode(xpe);
    String::from_utf8(decoded).ok()?.parse().ok()
}

impl FromStr for Magnet {
    type Err = MagnetError;

    /// Parse a magnet link.
    ///
    /// # Errors
    ///
    /// Fails if the prefix is wrong, or if the infohash is missing,
    /// misidentified, or unusable. Everything else is best-effort.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parameters are `key=value` joined by `&`. Anything without an `=`
        // is malformed and dropped.
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
                // `extend` over an Option adds the value if present, nothing
                // if the hint was malformed.
                "x.pe" => peer_addresses.extend(parse_x_pe(value)),
                // Unknown parameters (`xl`, `ws`, `kt`, anything custom).
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
