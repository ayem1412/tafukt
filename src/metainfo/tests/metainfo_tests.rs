use std::collections::BTreeMap;

use crate::metainfo::Metainfo;
use crate::metainfo::error::MetainfoError;
use crate::protocol::Bencode;

#[test]
fn metainfo_success_with_announce() {
    let mut top = BTreeMap::new();
    top.insert("announce".to_string(), Bencode::String(b"http://tracker.com".to_vec()));

    let mut info_dict = BTreeMap::new();
    info_dict.insert("name".to_string(), Bencode::String(b"test.iso".to_vec()));
    info_dict.insert("piece length".to_string(), Bencode::Integer(262144));
    info_dict.insert("pieces".to_string(), Bencode::String(vec![0; 20]));
    top.insert("info".to_string(), Bencode::Dictionary(info_dict));

    let bencode = Bencode::Dictionary(top);
    let result = Metainfo::try_from(bencode);

    assert!(result.is_ok());
    let m = result.unwrap();
    assert_eq!(m.announce, Some("http://tracker.com".to_string()));
}

#[test]
fn metainfo_success_without_announce_trackerless() {
    let mut top = BTreeMap::new();

    let mut info_dict = BTreeMap::new();
    info_dict.insert("name".to_string(), Bencode::String(b"test.iso".to_vec()));
    info_dict.insert("piece length".to_string(), Bencode::Integer(262144));
    info_dict.insert("pieces".to_string(), Bencode::String(vec![0; 20]));
    top.insert("info".to_string(), Bencode::Dictionary(info_dict));

    let bencode = Bencode::Dictionary(top);
    let result = Metainfo::try_from(bencode);

    assert!(result.is_ok());
    let m = result.unwrap();
    assert_eq!(m.announce, None);
}

#[test]
fn metainfo_missing_info() {
    let mut top = BTreeMap::new();
    top.insert("announce".to_string(), Bencode::String(b"http://tracker.com".to_vec()));

    let bencode = Bencode::Dictionary(top);
    assert!(matches!(
        Metainfo::try_from(bencode),
        Err(MetainfoError::MissingKey(k)) if k == "info"
    ));
}

#[test]
fn metainfo_top_level_not_dictionary() {
    let bencode = Bencode::Integer(42);
    assert!(matches!(Metainfo::try_from(bencode), Err(MetainfoError::NotADictionary)));
}

#[test]
fn metainfo_info_not_dictionary() {
    let mut top = BTreeMap::new();
    top.insert("info".to_string(), Bencode::Integer(123));

    let bencode = Bencode::Dictionary(top);
    assert!(matches!(Metainfo::try_from(bencode), Err(MetainfoError::NotADictionary)));
}
