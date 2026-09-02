//! Tests for the bencode encoder.
//!
//! Each case pins an exact byte sequence. That strictness is the point:
//! encoding must be the precise inverse of decoding, since a torrent's
//! identity is a hash of its original bytes and a single byte of difference
//! produces a hash no peer will accept.

use super::*;

// --- integers ----------------------------------------------------------

#[test]
fn integer_positive() {
    assert_eq!(encode(&Bencode::Integer(42)), b"i42e");
}

/// The minus sign comes from the number's own formatting — no special case.
#[test]
fn integer_negative() {
    assert_eq!(encode(&Bencode::Integer(-42)), b"i-42e");
}

#[test]
fn integer_zero() {
    assert_eq!(encode(&Bencode::Integer(0)), b"i0e");
}

/// The bounds of `i64`, in case the digits are ever written by hand rather
/// than by the standard formatter.
#[test]
fn integer_extremes() {
    assert_eq!(
        encode(&Bencode::Integer(i64::MAX)),
        b"i9223372036854775807e"
    );
    assert_eq!(
        encode(&Bencode::Integer(i64::MIN)),
        b"i-9223372036854775808e"
    );
}

// --- strings -----------------------------------------------------------

/// The length prefix is ASCII digits, not a binary number.
#[test]
fn string_normal() {
    assert_eq!(encode(&Bencode::String(b"spam")), b"4:spam");
}

#[test]
fn string_empty() {
    assert_eq!(encode(&Bencode::String(b"")), b"0:");
}

/// Byte strings hold arbitrary bytes, including nulls and invalid UTF-8 —
/// torrent files are full of both.
#[test]
fn string_arbitrary_bytes() {
    assert_eq!(
        encode(&Bencode::String(&[0xff, 0x00, 0xfe])),
        b"3:\xff\x00\xfe"
    );
}

// --- lists -------------------------------------------------------------

#[test]
fn list_empty() {
    assert_eq!(encode(&Bencode::List(vec![])), b"le");
}

#[test]
fn list_with_items() {
    let value = Bencode::List(vec![Bencode::String(b"spam"), Bencode::Integer(42)]);

    assert_eq!(encode(&value), b"l4:spami42ee");
}

// --- dictionaries ------------------------------------------------------

#[test]
fn dictionary_empty() {
    assert_eq!(encode(&Bencode::Dictionary(BTreeMap::new())), b"de");
}

/// Bencode requires keys in sorted byte order. The `BTreeMap` provides that
/// for free — this test is what would fail if the value type ever moved to a
/// hash map, which is why the decoder must keep using a `BTreeMap` too.
#[test]
fn dictionary_sorts_keys() {
    let mut entries = BTreeMap::new();
    entries.insert(b"spam".as_slice(), Bencode::String(b"eggs"));
    entries.insert(b"cow".as_slice(), Bencode::String(b"moo"));

    assert_eq!(
        encode(&Bencode::Dictionary(entries)),
        b"d3:cow3:moo4:spam4:eggse"
    );
}

/// Sorting is by raw bytes, not by length or by any locale rule —
/// `"a"` precedes `"ab"` precedes `"b"`.
#[test]
fn dictionary_sorts_bytewise() {
    let mut entries = BTreeMap::new();
    entries.insert(b"b".as_slice(), Bencode::Integer(3));
    entries.insert(b"ab".as_slice(), Bencode::Integer(2));
    entries.insert(b"a".as_slice(), Bencode::Integer(1));

    assert_eq!(
        encode(&Bencode::Dictionary(entries)),
        b"d1:ai1e2:abi2e1:bi3ee"
    );
}

// --- nesting -----------------------------------------------------------

#[test]
fn nested() {
    let inner = Bencode::List(vec![Bencode::String(b"a"), Bencode::String(b"b")]);

    let mut entries = BTreeMap::new();
    entries.insert(b"list".as_slice(), inner);

    assert_eq!(encode(&Bencode::Dictionary(entries)), b"d4:listl1:a1:bee");
}

// --- appending ---------------------------------------------------------

/// `encode_into` appends rather than replacing, so several values can share
/// one buffer without an allocation each.
#[test]
fn encode_into_appends() {
    let mut out = Vec::new();

    encode_into(&Bencode::Integer(1), &mut out);
    encode_into(&Bencode::Integer(2), &mut out);

    assert_eq!(out, b"i1ei2e");
}
