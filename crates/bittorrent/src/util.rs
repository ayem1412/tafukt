use std::{collections::BTreeMap, num::ParseIntError, path::PathBuf};

use bencode::bencode::Bencode;

use crate::metainfo::MetainfoError;

#[derive(Debug, thiserror::Error)]
pub enum HexError {
    #[error("hex string must have an even number of characters")]
    OddLength,
    #[error("invalid hex character: {0:?}")]
    InvalidCharacter(char),
}

pub type Dict<'a> = BTreeMap<&'a [u8], Bencode<'a>>;

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

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn hex_value(byte: u8) -> Result<u8, HexError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        other => Err(HexError::InvalidCharacter(other as char)),
    }
}

pub fn decode_hex_array<const N: usize>(s: &str) -> Result<[u8; N], HexError> {
    let bytes = s.as_bytes();

    if bytes.len() != N * 2 {
        return Err(HexError::OddLength);
    }

    let mut out = [0u8; N];
    for (slot, pair) in out.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *slot = hex_value(pair[0])? << 4 | hex_value(pair[1])?;
    }

    Ok(out)
}

pub fn percent_decode(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                decoded.push(b' ');
                i += 1;
            }

            b'%' if i + 3 <= bytes.len() => {
                match (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                    (Ok(high), Ok(low)) => {
                        decoded.push(high << 4 | low);
                        i += 3;
                    }
                    _ => {
                        decoded.push(b'%');
                        i += 1;
                    }
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
