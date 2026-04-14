use std::collections::BTreeMap;
use std::sync::OnceLock;

use bytes::Bytes;
use sha1::{Digest, Sha1};

use crate::metainfo::error::MetainfoError;
use crate::metainfo::file_layout::FileLayout;
use crate::metainfo::info_dictionary_file::InfoDictionaryFile;
use crate::metainfo::util;
use crate::protocol::{Bencode, encoder};

static INFO_HASH: OnceLock<Result<Bytes, MetainfoError>> = OnceLock::new();

/// Represents the `info` dictionary.
#[derive(Debug)]
pub struct InfoDictionary {
    /// The name key maps to a UTF-8 encoded string which is the suggested name to save the file (or
    /// directory) as. It is purely advisory.
    pub name: String,

    /// piece length maps to the number of bytes in each piece the file is split into.
    /// For the purposes of transfer, files are split into fixed-size pieces which are all the same
    /// length except for possibly the last one which may be truncated.
    /// piece length is almost always a power of two,
    /// most commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    pub piece_length: u64,

    /// pieces maps to a string whose length is a multiple of 20.
    /// It is to be subdivided into strings of length 20,
    /// each of which is the SHA1 hash of the piece at the corresponding index.
    pieces: Bytes,

    /// There is also a key `length` or a key `files`, but not both or neither.
    /// If length is present then the download represents a single file,
    /// otherwise it represents a set of files which go in a directory structure.
    file_layout: FileLayout,

    pub info_hash: [u8; 20],
}

impl InfoDictionary {
    pub fn piece_count(&self) -> usize {
        self.pieces.len() / 20
    }

    fn piece_len(&self, index: u32) -> u32 {
        let start = index as u64 * self.piece_length;
        let remaining = self.length().saturating_sub(start);

        remaining.min(self.piece_length) as u32
    }

    /// The length of the file, in bytes.
    /// In the single file case, length maps to the length of the file in bytes.
    pub fn length(&self) -> u64 {
        self.file_layout.length()
    }

    /// The SHA1 hash for piece `index`, or `None` if out of range.
    pub fn piece_hash(&self, index: usize) -> Option<[u8; 20]> {
        let start = index.checked_mul(20)?;
        let end = start.checked_mul(20)?;

        self.pieces.get(start..end)?.try_into().ok()
    }
}

impl TryFrom<Bencode> for InfoDictionary {
    type Error = MetainfoError;

    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let dict = match value {
            Bencode::Dictionary(d) => d,
            _ => return Err(MetainfoError::NotADictionary),
        };

        let name = util::extract_string_from_dict(&dict, "name")?;
        let piece_length = util::extract_integer_from_dict(&dict, "piece length")?;
        let pieces_raw = util::extract_bytes_from_dict(&dict, "pieces")?;

        if pieces_raw.len() % 20 != 0 {
            return Err(MetainfoError::InvalidPiecesLength);
        }

        let pieces: Bytes = pieces_raw.into();

        let file_layout = {
            let length = util::extract_optional_integer_from_dict(&dict, "length")?;
            let files = util::extract_optional_list_from_dict(&dict, "files", InfoDictionaryFile::try_from)?;

            match (length, files) {
                (Some(length), _) => FileLayout::SingleFile(length),
                (None, Some(files)) if !files.is_empty() => FileLayout::MultiFile(files),
                _ => return Err(MetainfoError::MissingFilesAndLength),
            }
        };

        let info_hash = compute_info_hash(&name, piece_length, &pieces, &file_layout);

        Ok(Self { name, piece_length, pieces, file_layout, info_hash })
    }
}

impl From<InfoDictionary> for Bencode {
    fn from(value: InfoDictionary) -> Self {
        let mut dict = BTreeMap::from([
            ("name".into(), Bencode::String(value.name.into_bytes())),
            ("piece length".into(), Bencode::Integer(value.piece_length as i64)),
            ("pieces".into(), Bencode::String(value.pieces.to_vec())),
        ]);

        match value.file_layout {
            FileLayout::SingleFile(length) => dict.insert("length".into(), Bencode::Integer(length as i64)),
            FileLayout::MultiFile(files) => dict.insert("files".into(), files_to_bencode(&files)),
        };

        Bencode::Dictionary(dict)
    }
}

fn compute_info_hash(name: &str, piece_length: u64, pieces: &Bytes, file_layout: &FileLayout) -> [u8; 20] {
    let mut dict = BTreeMap::from([
        ("name".into(), Bencode::String(name.as_bytes().to_vec())),
        ("piece length".into(), Bencode::Integer(piece_length as i64)),
        ("pieces".into(), Bencode::String(pieces.to_vec())),
    ]);

    match file_layout {
        FileLayout::SingleFile(length) => dict.insert("length".into(), Bencode::Integer(*length as i64)),
        FileLayout::MultiFile(files) => dict.insert("files".into(), files_to_bencode(files)),
    };

    Sha1::digest(encoder::encode(&Bencode::Dictionary(dict))).into()
}

fn files_to_bencode(files: &[InfoDictionaryFile]) -> Bencode {
    let list = files
        .iter()
        .map(|file| {
            let paths = file.path.iter().map(|component| Bencode::String(component.as_bytes().to_vec())).collect();

            Bencode::Dictionary(BTreeMap::from([
                ("length".into(), Bencode::Integer(file.length as i64)),
                ("path".into(), Bencode::List(paths)),
            ]))
        })
        .collect();

    Bencode::List(list)
}
