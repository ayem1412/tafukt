use crate::metainfo::error::MetainfoError;
use crate::metainfo::util;
use crate::protocol::Bencode;

/// The files list is the value files maps to, and is a list of dictionaries containing the
/// following keys:
#[derive(Debug)]
pub struct InfoDictionaryFile {
    /// The length of the file, in bytes.
    length: u64,

    /// A list of UTF-8 encoded strings corresponding to subdirectory names,
    /// the last of which is the actual file name (a zero length list is an error case).
    path: Vec<String>,
}

impl InfoDictionaryFile {
    pub fn new(length: u64, path: Vec<String>) -> Self {
        Self { length, path }
    }
}

impl TryFrom<Bencode> for InfoDictionaryFile {
    type Error = MetainfoError;

    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let dict = match value {
            Bencode::Dictionary(d) => d,
            _ => return Err(MetainfoError::NotADictionary),
        };

        let length = util::extract_integer_from_dict(&dict, "length")?;
        let path = util::extract_string_list_from_dict(&dict, "path")?;

        Ok(Self::new(length, path))
    }
}
