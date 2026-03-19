use std::collections::BTreeMap;

use crate::metainfo::error::MetainfoError;
use crate::protocol::Bencode;

pub fn extract_optional_string_from_dict(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
) -> Result<Option<String>, MetainfoError> {
    dict.get(key)
        .map(|bencode| match bencode {
            Bencode::String(bytes) => String::from_utf8(bytes.clone()).map_err(|_| MetainfoError::InvalidUtf8String),
            _ => Err(MetainfoError::WrongValueType(key.into())),
        })
        .transpose()
}

pub fn extract_string_from_dict(dict: &BTreeMap<String, Bencode>, key: &str) -> Result<String, MetainfoError> {
    match dict.get(key) {
        Some(Bencode::String(bytes)) => String::from_utf8(bytes.clone()).map_err(|_| MetainfoError::InvalidUtf8String),
        Some(_) => Err(MetainfoError::WrongValueType(key.into())),
        None => Err(MetainfoError::MissingKey(key.into())),
    }
}

pub fn extract_bytes_from_dict(dict: &BTreeMap<String, Bencode>, key: &str) -> Result<Vec<u8>, MetainfoError> {
    match dict.get(key) {
        Some(Bencode::String(bytes)) => Ok(bytes.clone()),
        Some(_) => Err(MetainfoError::WrongValueType(key.into())),
        None => Err(MetainfoError::MissingKey(key.into())),
    }
}

pub fn extract_integer_from_dict<T: TryFrom<i64>>(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
) -> Result<T, MetainfoError> {
    match dict.get(key) {
        Some(Bencode::Integer(value)) => value.clone().try_into().map_err(|_| MetainfoError::IntegerOverflow),
        Some(_) => Err(MetainfoError::WrongValueType(key.into())),
        None => Err(MetainfoError::MissingKey(key.into())),
    }
}

pub fn extract_bencode_from_dict(dict: &BTreeMap<String, Bencode>, key: &str) -> Result<Bencode, MetainfoError> {
    dict.get(key).cloned().ok_or_else(|| MetainfoError::MissingKey(key.into()))
}
