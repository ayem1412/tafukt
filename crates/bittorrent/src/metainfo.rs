//! Parsing `.torrent` files.
//!
//! A `.torrent` file is a bencode dictionary describing what to download:
//! the tracker to ask, the file layout, and a hash for every piece. This
//! module turns those bytes into a [`Metainfo`].
//!
//! ```no_run
//! use bittorrent::metainfo::Metainfo;
//!
//! let data = std::fs::read("example.torrent")?;
//! let torrent = Metainfo::from_bytes(&data)?;
//!
//! println!(
//!     "{} files, {} bytes",
//!     torrent.info.files.len(),
//!     torrent.info.length
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # One shape, not two
//!
//! The format describes single-file and multi-file torrents differently: one
//! has a `length` key, the other a `files` list. That distinction is resolved
//! during parsing. [`Info::files`] is always a flat list with cumulative
//! offsets, and a single-file torrent simply has one entry at offset 0 — so
//! nothing downstream has to branch on which shape the file used.
//!
//! # Validation
//!
//! Torrent files come from strangers, so parsing rejects:
//!
//! - paths containing `..`, separators, or null bytes, which could write outside the download
//!   directory
//! - negative file lengths, and offsets that overflow when summed
//! - a piece count that disagrees with the total size
//! - a `pieces` field whose length is not a multiple of 20

use std::{collections::BTreeMap, path::PathBuf};

use bencode::{
    bencode::{Bencode, BencodeError},
    decoder::{self, DecoderError},
};
use sha1::{Digest, Sha1};

use crate::util;

/// A decoded bencode dictionary, keyed by raw byte strings.
pub type Dict<'a> = BTreeMap<&'a [u8], Bencode<'a>>;

/// A `.torrent` file could not be parsed.
#[derive(Debug, thiserror::Error)]
pub enum MetainfoError {
    /// A required key was absent from a dictionary.
    #[error("could not find key: {0}")]
    KeyNotFound(&'static str),

    /// The `pieces` field's length was not a multiple of 20, so it cannot be
    /// a sequence of SHA-1 hashes.
    #[error("the length of the `pieces` field must divide evenly by 20")]
    InvalidPiecesLen,

    /// The `piece length` was zero, negative, or too large.
    #[error("invalid `piece length` value")]
    InvalidPieceLength,

    /// The number of piece hashes did not match what the total size implies.
    #[error("total size divided by piece length, rounded up, must equal the number of hashes")]
    InvalidPieceCount,

    /// A file length was negative, which no file can be.
    #[error("the length of the file must be positive")]
    NegativeLength,

    /// A path was empty, or contained `.`, `..`, a separator, or a null byte.
    ///
    /// Such a path could escape the download directory, so the whole torrent
    /// is rejected rather than sanitised — silently rewriting a path could
    /// collapse two files onto one another.
    #[error("the torrent has an invalid path")]
    InvalidPath,

    /// Summed file lengths exceeded what a `u64` can hold.
    #[error("offset overflow due to large file")]
    OffsetOverflow,

    /// Neither `length` nor `files` was present, so there is nothing to
    /// download.
    #[error("missing both `files` and `length` keys from the torrent file")]
    MissingFileInfo,

    /// Both `length` and `files` were present. The format allows exactly one.
    #[error("file includes both `files` and `length` keys")]
    BothFileKeys,

    /// The recorded span of the `info` dictionary fell outside the input.
    #[error("the recorded span of the `info` dictionary is out of bounds")]
    OutOfBoundsInfo,

    /// The file was not valid bencode.
    #[error(transparent)]
    DecoderError(#[from] DecoderError),

    /// A value was not the type the format requires.
    #[error(transparent)]
    BencodeError(#[from] BencodeError),
}

/// What the torrent describes: the files, how they are split, and their hashes.
#[derive(Debug)]
pub struct Info {
    /// Every file in order, with cumulative byte offsets.
    ///
    /// A single-file torrent has exactly one entry, at offset 0. Offsets are
    /// positions within the whole torrent treated as one continuous strip of
    /// bytes, which is what piece numbering is based on.
    pub files: Vec<FileEntry>,

    /// Total size of all files in bytes.
    pub length: u64,

    /// Suggested filename for a single-file torrent, or directory name for a
    /// multi-file one.
    ///
    /// Raw bytes, since torrent names have no guaranteed encoding.
    pub name: Vec<u8>,

    /// Bytes per piece, commonly 262,144 (256 KiB).
    ///
    /// Every piece is this size except the last, which holds whatever
    /// remains. That final short piece is not recorded anywhere — compute it
    /// from [`length`](Self::length) when you need it.
    pub piece_length: u32,

    /// One SHA-1 hash per piece, in order.
    ///
    /// Index by piece number: hash `n` verifies piece `n`. For multi-file
    /// torrents the pieces span the files as though they were concatenated,
    /// so a piece can cross a file boundary.
    pub pieces: Vec<[u8; 20]>,

    /// Whether the torrent is marked private (BEP 27).
    ///
    /// When true, peers may only be found through the trackers — DHT, peer
    /// exchange, and local peer discovery must all stay off. Private trackers
    /// rely on this to keep swarms closed, so ignoring it can get an account
    /// banned.
    pub private: bool,
}

/// One file within a torrent.
#[derive(Debug)]
pub struct FileEntry {
    /// Size of this file in bytes.
    pub length: u64,

    /// Path relative to the download directory, including the torrent's
    /// top-level name.
    pub path: PathBuf,

    /// Where this file begins in the torrent's continuous byte strip.
    ///
    /// The first file starts at 0; each subsequent one starts where the
    /// previous ended. Use this to map between piece numbers and files.
    pub offset: u64,
}

impl FileEntry {
    /// Assemble an entry. Private because offsets are only meaningful when
    /// computed in sequence across the whole file list.
    const fn new(length: u64, path: PathBuf, offset: u64) -> Self {
        Self {
            length,
            path,
            offset,
        }
    }
}

/// A parsed `.torrent` file.
#[derive(Debug)]
pub struct Metainfo {
    /// The primary tracker URL.
    ///
    /// Optional: DHT-only torrents have no tracker at all.
    pub announce: Option<String>,

    /// Tiers of backup trackers (BEP 12), if the torrent has any.
    ///
    /// Each inner list is a tier: try every tracker in a tier (shuffled)
    /// before falling through to the next one. When this is non-empty it
    /// supersedes [`announce`](Self::announce), which is only kept for old
    /// clients.
    pub announce_list: Vec<Vec<String>>,

    /// What the torrent contains and how it is divided.
    pub info: Info,

    /// SHA-1 of the raw bytes of the `info` dictionary, exactly as they
    /// appeared in the source file.
    ///
    /// This is the torrent's identity: it goes in the peer handshake,
    /// tracker announces, and DHT lookups. It is computed from the original
    /// byte span rather than a re-encoding of the parsed value — re-encoding
    /// can differ by a byte and produce a hash no peer will accept.
    pub info_hash: [u8; 20],
}

impl Metainfo {
    /// Parse the contents of a `.torrent` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes are not valid bencode, if a required
    /// field is missing or the wrong type, or if any of the validation rules
    /// described at the [module level](self) is violated.
    pub fn from_bytes(data: &[u8]) -> Result<Self, MetainfoError> {
        // Spans are needed because the infohash covers the `info`
        // dictionary's original bytes, not a re-encoding of them.
        let decoded = decoder::decode_dictionary_with_spans(data)?;
        let root = &decoded.root;

        let announce = util::get_opt_string_lossy(root, "announce")?;
        let announce_list = parse_announce_list(root)?;

        let info_dict = util::get_key(root, "info")?.as_dictionary()?;
        let info = parse_info(info_dict)?;

        let &(start, end) = decoded
            .spans
            .get(b"info".as_slice())
            .ok_or(MetainfoError::KeyNotFound("info"))?;

        let info_hash: [u8; 20] =
            Sha1::digest(data.get(start..end).ok_or(MetainfoError::OutOfBoundsInfo)?).into();

        Ok(Self {
            announce,
            announce_list,
            info,
            info_hash,
        })
    }
}

/// Read the optional `announce-list` into tiers of tracker URLs.
///
/// Absent in many torrents, in which case the list is empty.
fn parse_announce_list(root: &Dict) -> Result<Vec<Vec<String>>, MetainfoError> {
    let Some(tiers) = util::get_opt(root, "announce-list") else {
        return Ok(Vec::new());
    };

    tiers
        .as_list()?
        .iter()
        .map(|tier| {
            tier.as_list()?
                .iter()
                .map(|url| Ok(String::from_utf8_lossy(url.as_bytes()?).into_owned()))
                .collect()
        })
        .collect()
}

/// Read the `info` dictionary, validating it as it goes.
fn parse_info(info: &Dict) -> Result<Info, MetainfoError> {
    let name = util::get_key(info, "name")?.as_bytes()?;
    util::check_path_component(name)?;

    let piece_length = u32::try_from(util::get_key(info, "piece length")?.as_i64()?)
        .map_err(|_| MetainfoError::InvalidPieceLength)?;

    // Zero would mean dividing by zero everywhere piece maths happens.
    if piece_length == 0 {
        return Err(MetainfoError::InvalidPieceLength);
    }

    let private = match util::get_opt(info, "private") {
        Some(value) => value.as_i64()? != 0,
        None => false,
    };

    // `pieces` is one long byte string, not a list — 20 bytes per piece,
    // concatenated with no separators.
    let pieces = util::get_key(info, "pieces")?.as_bytes()?;
    if pieces.len() % 20 != 0 {
        return Err(MetainfoError::InvalidPiecesLen);
    }
    // The remainder is guaranteed empty by the check above.
    let pieces = pieces.as_chunks::<20>().0.to_vec();

    let (files, length) = parse_files(info, name)?;

    // Catches a torrent whose declared sizes and hash count disagree, which
    // would otherwise surface as a confusing failure mid-download.
    if length.div_ceil(u64::from(piece_length)) != pieces.len() as u64 {
        return Err(MetainfoError::InvalidPieceCount);
    }

    Ok(Info {
        files,
        length,
        name: name.to_vec(),
        piece_length,
        pieces,
        private,
    })
}

/// Build the flat file list with cumulative offsets, plus the total size.
///
/// Handles both torrent shapes — `length` for a single file, `files` for
/// many — and produces the same structure from either. A single-file torrent
/// yields exactly one entry at offset 0.
fn parse_files(info: &Dict, name: &[u8]) -> Result<(Vec<FileEntry>, u64), MetainfoError> {
    let single = util::get_opt(info, "length");
    let multi = util::get_opt(info, "files");

    // Exactly one of the two must be present.
    match (single, multi) {
        (Some(length), None) => {
            let length = util::as_u64(length)?;
            Ok((vec![FileEntry::new(length, util::to_path(name), 0)], length))
        }

        (None, Some(files)) => {
            let files = files.as_list()?;
            let mut entries = Vec::with_capacity(files.len());
            let mut offset = 0u64;

            for file in files {
                let dict = file.as_dictionary()?;
                let length = util::as_u64(util::get_key(dict, "length")?)?;

                // `path` is a list of components to join, not a single string.
                let components = util::get_key(dict, "path")?.as_list()?;
                if components.is_empty() {
                    return Err(MetainfoError::InvalidPath);
                }

                let mut path = util::to_path(name);
                for component in components {
                    let bytes = component.as_bytes()?;
                    util::check_path_component(bytes)?;
                    path.push(String::from_utf8_lossy(bytes).as_ref());
                }

                entries.push(FileEntry::new(length, path, offset));

                // A hostile torrent could claim sizes that overflow when summed.
                offset = offset
                    .checked_add(length)
                    .ok_or(MetainfoError::OffsetOverflow)?;
            }

            Ok((entries, offset))
        }

        (Some(_), Some(_)) => Err(MetainfoError::BothFileKeys),
        (None, None) => Err(MetainfoError::MissingFileInfo),
    }
}
