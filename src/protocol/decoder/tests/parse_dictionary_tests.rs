use std::collections::BTreeMap;

use crate::protocol::Bencode;
use crate::protocol::decoder::tests::decode;

#[test]
fn test_decode_dictionary_string() {
    let mut dictionary = BTreeMap::new();
    dictionary.insert("cow".to_string(), Bencode::String(b"moo".to_vec()));
    dictionary.insert("spam".to_string(), Bencode::String(b"eggs".to_vec()));
    assert_eq!(decode("d3:cow3:moo4:spam4:eggse"), Ok(Bencode::Dictionary(dictionary)))
}

#[test]
fn test_decode_dictionary_integer() {
    let mut dictionary = BTreeMap::new();
    dictionary.insert("eggs".to_string(), Bencode::Integer(10));
    assert_eq!(decode("d4:eggsi10ee"), Ok(Bencode::Dictionary(dictionary)))
}

#[test]
fn test_decode_dictionary_list() {
    let mut dictionary = BTreeMap::new();
    let list = vec![Bencode::String(b"a".to_vec()), Bencode::String(b"b".to_vec())];
    dictionary.insert("spam".to_string(), Bencode::List(list));
    assert_eq!(decode("d4:spaml1:a1:bee"), Ok(Bencode::Dictionary(dictionary)))
}

#[test]
fn test_decode_dictionary_mixed() {
    let mut dictionary = BTreeMap::new();
    dictionary.insert("eggs".to_string(), Bencode::Integer(10));
    dictionary.insert("spam".to_string(), Bencode::String(b"eggs".to_vec()));
    assert_eq!(decode("d4:eggsi10e4:spam4:eggse"), Ok(Bencode::Dictionary(dictionary)))
}
