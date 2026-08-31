use std::path::PathBuf;

use bencode::bencode::Bencode;

use crate::{
    hex,
    metainfo::{Dict, MetainfoError},
};

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

pub fn percent_decode(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'+' => {
                decoded.push(b' ');
                i += 1;
            }

            b'%' => {
                if let (Some(Ok(high)), Some(Ok(low))) = (
                    bytes.get(i + 1).copied().map(hex::hex_value),
                    bytes.get(i + 2).copied().map(hex::hex_value),
                ) {
                    decoded.push(high << 4 | low);
                    i += 3;
                } else {
                    decoded.push(b'%');
                    i += 1;
                }
            }

            other => {
                decoded.push(other);
                i += 1;
            }
        }
    }

    decoded
}
