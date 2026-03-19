use crate::metainfo::error::MetainfoError;
use crate::metainfo::info_dictionary::InfoDictionary;
use crate::protocol::Bencode;

#[cfg(test)]
mod tests;

mod error;
mod info_dictionary;
mod util;

/// Metainfo files (also known as .torrent files) are bencoded dictionaries.
/// NOTE: All strings in a .torrent file that contains text must be UTF-8 encoded.
#[derive(Debug)]
pub struct Metainfo {
    /// The URL of the tracker.
    announce: Option<String>,

    /// This maps to a dictionary.
    pub info: InfoDictionary,
}

impl Metainfo {
    fn new(announce: Option<String>, info: InfoDictionary) -> Self {
        Self { announce, info }
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
        let info = util::extract_bencode_from_dict(&dict, "info")?;

        let info = InfoDictionary::try_from(info)?;

        Ok(Self::new(announce, info))
    }
}
