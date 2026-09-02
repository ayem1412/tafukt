//! Encoding [`Bencode`] values back into bytes.
//!
//! Encoding is the exact inverse of [`decode`](crate::decoder::decode): a
//! value decoded from bytes and re-encoded reproduces those bytes
//! identically. That round-trip property is what makes it safe to compute a
//! `BitTorrent` infohash from decoded data.
//!
//! # Example
//!
//! ```
//! use bencode::{decoder::decode, encoder::encode};
//!
//! let original = b"d3:cow3:moo4:spam4:eggse";
//! let value = decode(original)?;
//!
//! assert_eq!(encode(&value), original);
//! # Ok::<(), bencode::decoder::DecoderError>(())
//! ```
//!
//! # Infallibility
//!
//! Nothing here returns a `Result`. Every [`Bencode`] value is representable
//! in bencode by construction, and writing to a `Vec` cannot fail, so there
//! is no error case to handle.

#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, io::Write};

use crate::bencode::Bencode;

/// Starting buffer size, chosen so small torrents never reallocate.
const INITIAL_CAPACITY: usize = 1024;

/// Encode a value into a new byte buffer.
///
/// Use [`encode_into`] instead to append to a buffer you already have.
///
/// # Example
///
/// ```
/// use bencode::{bencode::Bencode, encoder::encode};
///
/// assert_eq!(encode(&Bencode::Integer(42)), b"i42e");
/// assert_eq!(encode(&Bencode::String(b"spam")), b"4:spam");
/// ```
#[must_use]
pub fn encode(value: &Bencode) -> Vec<u8> {
    let mut out = Vec::with_capacity(INITIAL_CAPACITY);
    encode_into(value, &mut out);
    out
}

/// Encode a value, appending to an existing buffer.
///
/// Useful when building several values into one buffer, since it avoids the
/// intermediate allocation [`encode`] would make for each.
pub fn encode_into(value: &Bencode, out: &mut Vec<u8>) {
    match value {
        Bencode::Integer(number) => encode_integer(*number, out),
        Bencode::String(bytes) => encode_string(bytes, out),
        Bencode::List(items) => encode_list(items, out),
        Bencode::Dictionary(entries) => encode_dictionary(entries, out),
    }
}

/// Write an integer as `i`, its decimal digits, then `e`.
///
/// A negative sign comes from the number's own formatting, so `-42` becomes
/// `i-42e` with no special handling.
fn encode_integer(number: i64, out: &mut Vec<u8>) {
    out.push(b'i');
    // Writing to a Vec is infallible; the Result exists only to satisfy the
    // io::Write trait.
    let _ = write!(out, "{number}");
    out.push(b'e');
}

/// Write a byte string as its length in decimal, a `:`, then the bytes.
///
/// The length is written in ASCII digits, not as a binary number — bencode
/// spells all its numbers out as text.
fn encode_string(bytes: &[u8], out: &mut Vec<u8>) {
    let _ = write!(out, "{}", bytes.len());
    out.push(b':');
    out.extend_from_slice(bytes);
}

/// Write a list as `l`, each item in order, then `e`.
fn encode_list(items: &[Bencode], out: &mut Vec<u8>) {
    out.push(b'l');

    for item in items {
        encode_into(item, out);
    }

    out.push(b'e');
}

/// Write a dictionary as `d`, alternating keys and values, then `e`.
///
/// Bencode requires keys in sorted byte order. [`BTreeMap`] iterates in that
/// order already, so no sorting happens here — which is also why the decoder
/// must keep using a `BTreeMap` rather than a hash map.
fn encode_dictionary(entries: &BTreeMap<&[u8], Bencode>, out: &mut Vec<u8>) {
    out.push(b'd');

    for (key, value) in entries {
        encode_string(key, out);
        encode_into(value, out);
    }

    out.push(b'e');
}
