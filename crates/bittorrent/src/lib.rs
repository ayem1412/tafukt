//! `BitTorrent` metadata types.
//!
//! Reads the two ways a torrent can be described — a `.torrent` file or a
//! magnet link — into structures the rest of a client can use.
//!
//! Nothing here touches the network or needs an async runtime. That is a
//! deliberate constraint: it keeps this crate usable from any program,
//! whatever its runtime.
//!
//! # Quick start
//!
//! ```no_run
//! use bittorrent::{magnet::Magnet, metainfo::Metainfo};
//!
//! // From a file
//! let data = std::fs::read("example.torrent")?;
//! let torrent = Metainfo::from_bytes(&data)?;
//!
//! // From a magnet link
//! let magnet: Magnet = "magnet:?xt=urn:btih:...".parse()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # The infohash
//!
//! A torrent's identity is the SHA-1 of the raw bytes of its `info`
//! dictionary, exactly as they appeared in the file. It is computed from the
//! original byte span rather than a re-encoding, since re-encoding can differ
//! by a byte and produce a hash no peer will accept.
//!
//! Magnet links carry the same hash directly, written as 40 hex characters
//! or 32 base32 characters.
//!
//! # Names are bytes, not text
//!
//! Torrent filenames have no guaranteed encoding — real ones appear in
//! Shift-JIS, GBK, and others. They are kept as raw bytes and converted
//! lossily only for display, so nothing is silently mangled during parsing.
//!
//! # Hostile input
//!
//! Torrent files come from strangers. Parsing rejects paths that could escape
//! the download directory, negative or overflowing sizes, and metadata whose
//! piece count disagrees with its total size. Malformed input returns an
//! error rather than panicking.

/// Hex encoding and decoding.
pub mod hex;

/// Parsing magnet links.
pub mod magnet;

/// Parsing `.torrent` files.
pub mod metainfo;

/// Shared helpers for reading bencode dictionaries.
pub mod util;

pub use magnet::{Magnet, MagnetError};
pub use metainfo::{FileEntry, Info, Metainfo, MetainfoError};
