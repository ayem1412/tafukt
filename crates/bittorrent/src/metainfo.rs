use std::path::PathBuf;

use bencode::{
    bencode::BencodeError,
    decoder::{self, DecoderError},
};
use sha1::{Digest, Sha1};

use crate::util::{self, Dict};

#[derive(Debug, thiserror::Error)]
pub enum MetainfoError {
    #[error("could not find key: {0}")]
    KeyNotFound(&'static str),
    #[error("the length of the `pieces` field must divide evenly by 20")]
    InvalidPiecesLen,
    #[error("invalid `piece length` value")]
    InvalidPieceLength,
    #[error("total size divided by piece length, rounded up, must equal the number of hashes")]
    InvalidPieceCount,
    #[error("the length of the file must be positive")]
    NegativeLength,
    #[error("the torrent has an invalid path")]
    InvalidPath,
    #[error("offset overflow due to large file")]
    OffsetOverflow,
    #[error("missing both `files` and `length` keys from the torrent file")]
    MissingFileInfo,
    #[error("file includes both `files` and `length` keys")]
    BothFileKeys,
    #[error(transparent)]
    DecoderError(#[from] DecoderError),
    #[error(transparent)]
    BencodeError(#[from] BencodeError),
}

#[derive(Debug)]
pub struct Info {
    /// all files in order, with cumulative offsets; single-file torrents have exactly one entry.
    pub files: Vec<FileEntry>,
    /// size of file(s) in bytes.
    pub length: u64,
    /// suggested filename where the file is to be saved (if one file)/suggested directory name
    /// where the files are to be saved (if multiple files).
    pub name: Vec<u8>,
    /// number of bytes per piece. This is commonly 256 KiB = 262,144 B.
    pub piece_length: u32,
    /// a hash list, i.e., a concatenation of each piece's SHA-1 hash. As SHA-1 returns a 160-bit
    /// hash, pieces will be a string whose length is a multiple of 20 bytes. If the torrent
    /// contains multiple files, the pieces are formed by concatenating the files in the order they
    /// appear in the files dictionary (i.e., all pieces in the torrent are the full piece length
    /// except for the last piece, which may be shorter).
    pub pieces: Vec<[u8; 20]>,
    /// whether the torrent is marked private (BEP 27).
    ///
    /// when true, peers may only be found through the trackers —
    /// DHT, peer exchange, and local peer discovery must all stay off. Private
    /// trackers rely on this to keep swarms closed, so ignoring it can get an
    /// account banned.
    pub private: bool,
}

#[derive(Debug)]
pub struct FileEntry {
    /// size of the file in bytes.
    pub length: u64,
    /// relative to the download directory.
    pub path: PathBuf,
    /// where this file starts in the whole-torrent strip.
    pub offset: u64,
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

#[derive(Debug)]
pub struct Metainfo {
    /// the URL of the tracker.
    pub announce: Option<String>,
    /// tiers of backup trackers (BEP 12), if the torrent has any.
    ///
    /// each inner list is a tier: try every tracker in a tier (shuffled) before
    /// falling through to the next one. When this is non-empty it supersedes
    /// [`announce`], which is only kept for old clients.
    pub announce_list: Vec<Vec<String>>,
    /// this maps to a dictionary whose keys are very dependent on whether one or more files are
    /// being shared.
    pub info: Info,
    /// SHA-1 of the raw bytes of the `info` dictionary, exactly as they appeared
    /// in the source file.
    ///
    /// this is the torrent's identity: it goes in the handshake, tracker
    /// announces, and DHT lookups. It must be computed from the original byte
    /// span rather than a re-encoding of the parsed value — re-encoding can
    /// differ by a byte and produce a hash no peer will accept.
    pub info_hash: [u8; 20],
}

impl Metainfo {
    pub fn from_bytes(data: &[u8]) -> Result<Self, MetainfoError> {
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
        let info_hash: [u8; 20] = Sha1::digest(&data[start..end]).into();

        Ok(Self {
            announce,
            announce_list,
            info,
            info_hash,
        })
    }
}

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

fn parse_info(info: &Dict) -> Result<Info, MetainfoError> {
    let name = util::get_key(info, "name")?.as_bytes()?;
    util::check_path_component(name)?;

    let piece_length = u32::try_from(util::get_key(info, "piece length")?.as_i64()?)
        .map_err(|_| MetainfoError::InvalidPieceLength)?;
    if piece_length == 0 {
        return Err(MetainfoError::InvalidPieceLength);
    }

    let private = match util::get_opt(info, "private") {
        Some(value) => value.as_i64()? != 0,
        None => false,
    };

    let pieces = util::get_key(info, "pieces")?.as_bytes()?;
    if pieces.len() % 20 != 0 {
        return Err(MetainfoError::InvalidPiecesLen);
    }
    // the remainder is guaranteed empty by the check above.
    let pieces = pieces.as_chunks::<20>().0.to_vec();

    let (files, length) = parse_files(info, name)?;

    // total size must account for exactly the number of hashes present.
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

/// builds the flat file list with cumulative offsets, plus the total size.
/// a single-file torrent yields exactly one entry at offset 0.
fn parse_files(info: &Dict, name: &[u8]) -> Result<(Vec<FileEntry>, u64), MetainfoError> {
    let single = util::get_opt(info, "length");
    let multi = util::get_opt(info, "files");

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
