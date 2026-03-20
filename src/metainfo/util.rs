use std::collections::BTreeMap;

use crate::metainfo::error::MetainfoError;
use crate::protocol::Bencode;

pub fn extract_optional_integer_from_dict<T: TryFrom<i64>>(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
) -> Result<Option<T>, MetainfoError> {
    dict.get(key)
        .map(|bencode| match bencode {
            Bencode::Integer(value) => value.clone().try_into().map_err(|_| MetainfoError::IntegerOverflow),
            _ => Err(MetainfoError::WrongValueType(key.into())),
        })
        .transpose()
}

pub fn extract_optional_list_from_dict<F, T>(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
    converter: F,
) -> Result<Option<Vec<T>>, MetainfoError>
where
    F: Fn(Bencode) -> Result<T, MetainfoError>,
{
    dict.get(key)
        .map(|bencode| match bencode {
            Bencode::List(list) => list.clone().into_iter().map(converter).collect(),
            _ => Err(MetainfoError::WrongValueType(key.into())),
        })
        .transpose()
}

pub fn extract_list_from_dict<F, T>(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
    converter: F,
) -> Result<Vec<T>, MetainfoError>
where
    F: Fn(Bencode) -> Result<T, MetainfoError>,
{
    extract_bencode_list_from_dict(dict, key)?.into_iter().map(converter).collect::<Result<Vec<_>, _>>()
}

pub fn extract_string_list_from_dict(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
) -> Result<Vec<String>, MetainfoError> {
    match dict.get(key) {
        Some(Bencode::List(list)) => list
            .iter()
            .map(|el| match el {
                Bencode::String(bytes) => {
                    String::from_utf8(bytes.clone()).map_err(|_| MetainfoError::InvalidUtf8String)
                },
                _ => Err(MetainfoError::WrongValueType(key.into())),
            })
            .collect(),
        Some(_) => Err(MetainfoError::WrongValueType(key.into())),
        None => Err(MetainfoError::MissingKey(key.into())),
    }
}

pub fn extract_bencode_list_from_dict(
    dict: &BTreeMap<String, Bencode>,
    key: &str,
) -> Result<Vec<Bencode>, MetainfoError> {
    match dict.get(key) {
        Some(Bencode::List(list)) => Ok(list.clone()),
        Some(_) => Err(MetainfoError::WrongValueType(key.into())),
        None => Err(MetainfoError::MissingKey(key.into())),
    }
}

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
