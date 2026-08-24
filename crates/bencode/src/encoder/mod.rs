#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, io::Write};

use crate::bencode::Bencode;

pub fn encode(bencode: &Bencode) -> Vec<u8> {
    let mut out = Vec::with_capacity(1024);
    encode_into(bencode, &mut out);
    out
}

pub fn encode_into(bencode: &Bencode, out: &mut Vec<u8>) {
    match bencode {
        Bencode::Integer(number) => encode_integer(*number, out),
        Bencode::String(bytes) => encode_string(bytes, out),
        Bencode::List(items) => encode_list(items, out),
        Bencode::Dictionary(dict) => encode_dictionary(dict, out),
    }
}

fn encode_integer(value: i64, out: &mut Vec<u8>) {
    out.push(b'i');
    let _ = write!(out, "{value}");
    out.push(b'e');
}

fn encode_string(value: &[u8], out: &mut Vec<u8>) {
    let _ = write!(out, "{}", value.len());
    out.push(b':');
    out.extend_from_slice(value);
}

fn encode_list(items: &[Bencode], out: &mut Vec<u8>) {
    out.push(b'l');

    for item in items {
        encode_into(item, out);
    }

    out.push(b'e');
}

fn encode_dictionary(dict: &BTreeMap<&[u8], Bencode>, out: &mut Vec<u8>) {
    out.push(b'd');

    for (key, value) in dict {
        encode_string(key, out);
        encode_into(value, out);
    }

    out.push(b'e');
}
