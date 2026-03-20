use crate::metainfo::error::MetainfoError;
use crate::metainfo::info_dictionary_file::InfoDictionaryFile;
use crate::metainfo::util;
use crate::protocol::Bencode;

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
    pieces: Vec<u8>,

    /**
     * There is also a key `length` or a key `files`, but not both or neither.
     * If length is present then the download represents a single file,
     * otherwise it represents a set of files which go in a directory structure.
     */

    /// The length of the file, in bytes.
    /// In the single file case, length maps to the length of the file in bytes.
    length: Option<u64>,

    /// For the purposes of the other keys,
    /// the multi-file case is treated as only having a single file by concatenating the files
    /// in the order they appearin the files list.
    /// The files list is the value files maps to.
    files: Option<Vec<InfoDictionaryFile>>,
}

impl InfoDictionary {
    fn new(
        name: String,
        piece_length: u64,
        pieces: Vec<u8>,
        length: Option<u64>,
        files: Option<Vec<InfoDictionaryFile>>,
    ) -> Self {
        Self { name, piece_length, pieces, length, files }
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.len() / 20
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
        let pieces = util::extract_bytes_from_dict(&dict, "pieces")?;

        if pieces.len() % 20 != 0 {
            return Err(MetainfoError::InvalidPiecesLength);
        }

        let length = util::extract_optional_integer_from_dict(&dict, "length")?;
        let files = util::extract_optional_list_from_dict(&dict, "files", InfoDictionaryFile::try_from)?;

        if length.is_none() && files.as_ref().map_or(true, |files| files.is_empty()) {
            return Err(MetainfoError::MissingFilesAndLength);
        }

        Ok(Self::new(name, piece_length, pieces, length, files))
    }
}
