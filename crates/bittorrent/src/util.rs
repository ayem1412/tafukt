use std::{collections::BTreeMap, path::PathBuf};

use bencode::bencode::Bencode;

use crate::metainfo::MetainfoError;

type Dict<'a> = BTreeMap<&'a [u8], Bencode<'a>>;

pub fn get_key<'a>(
    dict: &'a Dict<'a>,
    key: &'static str,
) -> Result<&'a Bencode<'a>, MetainfoError> {
    dict.get(key.as_bytes())
        .ok_or(MetainfoError::KeyNotFound(key))
}

pub fn get_opt<'a>(dict: &'a Dict<'a>, key: &'static str) -> Option<&'a Bencode<'a>> {
    dict.get(key.as_bytes())
}

pub fn get_opt_string_lossy<'a>(
    dict: &'a Dict<'a>,
    key: &'static str,
) -> Result<Option<String>, MetainfoError> {
    match dict.get(key.as_bytes()) {
        Some(value) => Ok(Some(
            String::from_utf8_lossy(value.as_bytes()?).into_owned(),
        )),
        None => Ok(None),
    }
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

pub fn check_path_component(component: &[u8]) -> Result<(), MetainfoError> {
    if component.is_empty()
        || component == b"."
        || component == b".."
        || component.contains(&b'/')
        || component.contains(&b'\\')
        || component.contains(&0)
    {
        return Err(MetainfoError::InvalidPath);
    }

    Ok(())
}

pub fn as_u64(value: &Bencode) -> Result<u64, MetainfoError> {
    u64::try_from(value.as_i64()?).map_err(|_| MetainfoError::NegativeLength)
}

pub fn to_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}
