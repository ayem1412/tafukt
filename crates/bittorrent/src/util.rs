use std::collections::BTreeMap;

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
