use std::path::PathBuf;

use bencode::{
    bencode::BencodeError,
    decoder::{self, DecoderError},
};
use sha1::{Digest, Sha1};

use crate::util::{self, check_path_component};

#[derive(Debug, thiserror::Error)]
pub enum MetainfoError {
    #[error("could not find key: {0}")]
    KeyNotFound(&'static str),
    #[error("the length of the `pieces` field must divide evenly by 20")]
    InvalidPiecesLength,
    #[error("the length of the file must be positive")]
    NegativeLength,
    #[error("the torrent has an invalid path")]
    InvalidPath,
    #[error("offset overflow due to large file")]
    OffsetOverflow,
    #[error("missing both `files` and `length` keys from the torrent file")]
    MissingFileInfo,
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

#[derive(Debug)]
pub struct FileEntry {
    /// size of the file in bytes.
    length: u64,
    /// relative to the download directory.
    path: PathBuf,
    /// where this file starts in the whole-torrent strip.
    offset: u64,
}

impl FileEntry {
    fn new(length: u64, path: PathBuf, offset: u64) -> Self {
        Self {
            length,
            path,
            offset,
        }
    }
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
    pub fn from_bytes(data: &[u8]) -> Result<(), MetainfoError> {
        let decoded_root = decoder::decode_dictionary_with_spans(data)?;
        let root = &decoded_root.root;

        let announce = util::get_opt_string_lossy(root, "announce")?;

        let info = util::get_key(root, "info")?.as_dictionary()?;

        let name = util::get_key(info, "name")?.as_bytes()?;
        util::check_path_component(name)?;

        let piece_length = util::get_key(info, "piece length")?.as_i64()? as u32;
        let private = if let Some(private) = util::get_opt(info, "private") {
            private.as_i64()? != 0
        } else {
            false
        };

        let pieces = util::get_key(info, "pieces")?.as_bytes()?;
        if pieces.len() % 20 != 0 {
            return Err(MetainfoError::InvalidPiecesLength);
        }
        let pieces: Vec<[u8; 20]> = pieces
            .chunks_exact(20)
            .map(|chunk| {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(chunk);
                hash
            })
            .collect();

        let mut file_entries = vec![];
        if let Some(length) = util::get_opt(info, "length") {
            file_entries.push(FileEntry::new(
                util::as_u64(length)?,
                util::to_path(name),
                0,
            ));
        } else if let Some(files) = util::get_opt(info, "files") {
            let files = files.as_list()?;
            file_entries.reserve(files.len());

            let mut offset = 0;

            for file in files {
                let dict = file.as_dictionary()?;
                let length = util::as_u64(util::get_key(dict, "length")?)?;

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

                file_entries.push(FileEntry::new(length, path, offset));

                // very unlikely to happen but just to be safe.
                offset = offset
                    .checked_add(length)
                    .ok_or(MetainfoError::OffsetOverflow)?;
            }
        } else {
            return Err(MetainfoError::MissingFileInfo);
        }

        let &(start, end) = decoded_root
            .spans
            .get(b"info".as_slice())
            .ok_or(MetainfoError::KeyNotFound("info"))?;

        let info_hash: [u8; 20] = Sha1::digest(&data[start..end]).into();
        Ok(())
    }
}
