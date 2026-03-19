use std::collections::BTreeMap;

use crate::metainfo::error::MetainfoError;
use crate::metainfo::util;
use crate::protocol::Bencode;

#[test]
fn extract_optional_string_missing_key() {
    let dict: BTreeMap<String, Bencode> = BTreeMap::new();
    assert_eq!(util::extract_optional_string_from_dict(&dict, "announce"), Ok(None));
}

#[test]
fn extract_optional_string_present_valid() {
    let mut dict = BTreeMap::new();
    dict.insert("announce".to_string(), Bencode::String(b"http://example.com".to_vec()));

    assert_eq!(util::extract_optional_string_from_dict(&dict, "announce"), Ok(Some("http://example.com".to_string())));
}

#[test]
fn extract_optional_string_invalid_utf8() {
    let mut dict = BTreeMap::new();
    dict.insert("bad".to_string(), Bencode::String(vec![0xFF]));

    assert!(matches!(util::extract_optional_string_from_dict(&dict, "bad"), Err(MetainfoError::InvalidUtf8String)));
}

#[test]
fn extract_optional_string_wrong_type() {
    let mut dict = BTreeMap::new();
    dict.insert("announce".to_string(), Bencode::Integer(123));

    assert!(matches!(
        util::extract_optional_string_from_dict(&dict, "announce"),
        Err(MetainfoError::WrongValueType(k)) if k == "announce"
    ));
}

#[test]
fn extract_string_from_dict_success() {
    let mut dict = BTreeMap::new();
    dict.insert("name".to_string(), Bencode::String(b"ubuntu.iso".to_vec()));

    assert_eq!(util::extract_string_from_dict(&dict, "name"), Ok("ubuntu.iso".to_string()));
}

#[test]
fn extract_string_missing_key() {
    let dict: BTreeMap<String, Bencode> = BTreeMap::new();
    assert!(matches!(
        util::extract_string_from_dict(&dict, "name"),
        Err(MetainfoError::MissingKey(k)) if k == "name"
    ));
}

#[test]
fn extract_bytes_from_dict_success() {
    let mut dict = BTreeMap::new();
    let data = vec![1u8, 2, 3];
    dict.insert("pieces".to_string(), Bencode::String(data.clone()));

    assert_eq!(util::extract_bytes_from_dict(&dict, "pieces"), Ok(data));
}

#[test]
fn extract_bytes_wrong_type() {
    let mut dict = BTreeMap::new();
    dict.insert("pieces".to_string(), Bencode::Integer(42));

    assert!(matches!(
        util::extract_bytes_from_dict(&dict, "pieces"),
        Err(MetainfoError::WrongValueType(k)) if k == "pieces"
    ));
}

#[test]
fn extract_integer_from_dict_success_u64() {
    let mut dict = BTreeMap::new();
    dict.insert("piece length".to_string(), Bencode::Integer(262144));

    let result: Result<u64, _> = util::extract_integer_from_dict(&dict, "piece length");
    assert_eq!(result, Ok(262144));
}

#[test]
fn extract_integer_negative_to_u64_fails() {
    let mut dict = BTreeMap::new();
    dict.insert("piece length".to_string(), Bencode::Integer(-1));

    let result: Result<u64, _> = util::extract_integer_from_dict(&dict, "piece length");
    assert!(matches!(result, Err(MetainfoError::IntegerOverflow)));
}

#[test]
fn extract_bencode_from_dict_success() {
    let mut dict = BTreeMap::new();
    let inner = Bencode::Integer(42);
    dict.insert("info".to_string(), inner.clone());

    assert_eq!(util::extract_bencode_from_dict(&dict, "info"), Ok(inner));
}

#[test]
fn extract_bencode_missing_key() {
    let dict: BTreeMap<String, Bencode> = BTreeMap::new();
    assert!(matches!(
        util::extract_bencode_from_dict(&dict, "info"),
        Err(MetainfoError::MissingKey(k)) if k == "info"
    ));
}
