use crate::metainfo::error::MetainfoError;
use crate::metainfo::info_dictionary::InfoDictionary;
use crate::protocol::Bencode;

#[cfg(test)]
mod tests;

mod error;
mod file_layout;
pub mod info_dictionary;
mod info_dictionary_file;
mod util;

/// Metainfo files (also known as .torrent files) are bencoded dictionaries.
/// NOTE: All strings in a .torrent file that contains text must be UTF-8 encoded.
#[derive(Debug)]
pub struct Metainfo {
    /// The URL of the tracker.
    pub announce: Option<String>,

    /// This maps to a dictionary.
    pub info: InfoDictionary,
}

impl Metainfo {
    pub fn info_hash(&self) -> [u8; 20] {
        self.info.info_hash
    }
}

impl TryFrom<Bencode> for Metainfo {
    type Error = MetainfoError;

    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let dict = match value {
            Bencode::Dictionary(d) => d,
            _ => return Err(MetainfoError::NotADictionary),
        };

        let announce = util::extract_optional_string_from_dict(&dict, "announce")?;
        let info = util::extract_bencode_from_dict(&dict, "info").and_then(InfoDictionary::try_from)?;

        Ok(Self { announce, info })
    }
}
