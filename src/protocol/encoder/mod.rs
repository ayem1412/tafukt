use std::collections::BTreeMap;

use crate::protocol::Bencode;

pub fn encode(bencode: &Bencode) -> Vec<u8> {
    match bencode {
        Bencode::Integer(value) => encode_integer(*value),
        Bencode::List(items) => encode_list(items),
        Bencode::String(bytes) => encode_string(bytes),
        Bencode::Dictionary(dict) => encode_dictionary(dict),
    }
}

fn encode_integer(value: i64) -> Vec<u8> {
    format!("i{value}e").into_bytes()
}

fn encode_list(items: &[Bencode]) -> Vec<u8> {
    let mut result = vec![b'l'];
    result.extend(items.iter().flat_map(encode));
    result.push(b'e');
    result
}

fn encode_string(bytes: &[u8]) -> Vec<u8> {
    let mut result = format!("{}:", bytes.len()).into_bytes();
    result.extend_from_slice(bytes);
    result
}

fn encode_dictionary(dict: &BTreeMap<String, Bencode>) -> Vec<u8> {
    let mut result = vec![b'd'];
    result.extend(dict.iter().flat_map(|(k, v)| encode_string(k.as_bytes()).into_iter().chain(encode(v))));
    result.push(b'e');
    result
}
