use std::path::PathBuf;

use bencode::bencode::BencodeError;
use bencode::decoder::{self, DecoderError};

use crate::util;

#[derive(Debug, thiserror::Error)]
pub enum MetainfoError {
    #[error("could not find key: {0}")]
    KeyNotFound(&'static str),
    #[error(transparent)]
    DecoderError(#[from] DecoderError),
    #[error(transparent)]
    BencodeError(#[from] BencodeError),
}

struct Info {
    /// all files in order, with cumulative offsets; single-file torrents have exactly one entry.
    files: Vec<FileEntry>,
    /// size of file(s) in bytes.
    length: u64,
    /// suggested filename where the file is to be saved (if one file)/suggested directory name
    /// where the files are to be saved (if multiple files).
    name: Vec<u8>,
    /// number of bytes per piece. This is commonly 256 KiB = 262,144 B.
    piece_length: u32,
    /// a hash list, i.e., a concatenation of each piece's SHA-1 hash. As SHA-1 returns a 160-bit
    /// hash, pieces will be a string whose length is a multiple of 20 bytes. If the torrent
    /// contains multiple files, the pieces are formed by concatenating the files in the order they
    /// appear in the files dictionary (i.e., all pieces in the torrent are the full piece length
    /// except for the last piece, which may be shorter).
    pieces: Vec<[u8; 20]>,
    /// whether the torrent is marked private (BEP 27).
    ///
    /// when true, peers may only be found through the trackers —
    /// DHT, peer exchange, and local peer discovery must all stay off. Private
    /// trackers rely on this to keep swarms closed, so ignoring it can get an
    /// account banned.
    private: bool,
}

struct FileEntry {
    /// size of the file in bytes.
    length: u64,
    /// relative to the download directory.
    path: PathBuf,
    /// where this file starts in the whole-torrent strip.
    offset: u64,
}

pub struct Metainfo {
    /// the URL of the tracker.
    announce: Option<String>,
    /// tiers of backup trackers (BEP 12), if the torrent has any.
    ///
    /// each inner list is a tier: try every tracker in a tier (shuffled) before
    /// falling through to the next one. When this is non-empty it supersedes
    /// [`announce`], which is only kept for old clients.
    announce_list: Vec<Vec<String>>,
    /// this maps to a dictionary whose keys are very dependent on whether one or more files are
    /// being shared.
    info: Info,
    /// SHA-1 of the raw bytes of the `info` dictionary, exactly as they appeared
    /// in the source file.
    ///
    /// this is the torrent's identity: it goes in the handshake, tracker
    /// announces, and DHT lookups. It must be computed from the original byte
    /// span rather than a re-encoding of the parsed value — re-encoding can
    /// differ by a byte and produce a hash no peer will accept.
    info_hash: [u8; 20],
}

impl Metainfo {
    fn from_bytes(data: &[u8]) -> Result<(), MetainfoError> {
        let decoded_root = decoder::decode_dictionary_with_spans(data)?;
        let root = &decoded_root.root;

        let announce = util::get_opt_string_lossy(root, "announce")?;
        Ok(())
    }
}
