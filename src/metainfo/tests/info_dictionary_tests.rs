use std::collections::BTreeMap;

use crate::metainfo::error::MetainfoError;
use crate::metainfo::info_dictionary::InfoDictionary;
use crate::protocol::Bencode;

#[test]
fn info_dictionary_success_single_file() {
    let mut info_dict = BTreeMap::new();
    info_dict.insert("name".to_string(), Bencode::String(b"test.iso".to_vec()));
    info_dict.insert("piece length".to_string(), Bencode::Integer(262144));
    info_dict.insert("pieces".to_string(), Bencode::String(vec![0; 20])); // 1 piece

    let bencode = Bencode::Dictionary(info_dict);
    let result = InfoDictionary::try_from(bencode);

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.name, "test.iso");
    assert_eq!(info.piece_length, 262144);
    assert_eq!(info.piece_count(), 1);
}

#[test]
fn info_dictionary_pieces_not_multiple_of_20() {
    let mut info_dict = BTreeMap::new();
    info_dict.insert("name".to_string(), Bencode::String(b"test.iso".to_vec()));
    info_dict.insert("piece length".to_string(), Bencode::Integer(262144));
    info_dict.insert("pieces".to_string(), Bencode::String(vec![0; 21]));

    let bencode = Bencode::Dictionary(info_dict);
    assert!(matches!(InfoDictionary::try_from(bencode), Err(MetainfoError::InvalidPiecesLength)));
}

#[test]
fn info_dictionary_missing_name() {
    let mut info_dict = BTreeMap::new();
    info_dict.insert("piece length".to_string(), Bencode::Integer(262144));
    info_dict.insert("pieces".to_string(), Bencode::String(vec![0; 20]));

    let bencode = Bencode::Dictionary(info_dict);
    assert!(matches!(
        InfoDictionary::try_from(bencode),
        Err(MetainfoError::MissingKey(k)) if k == "name"
    ));
}

#[test]
fn info_dictionary_piece_count() {
    let info = InfoDictionary::new("test".to_string(), 262144, vec![0; 60]);
    assert_eq!(info.piece_count(), 3);

    let empty = InfoDictionary::new("test".to_string(), 262144, vec![]);
    assert_eq!(empty.piece_count(), 0);
}
