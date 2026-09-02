//! Tests for the bencode decoder.
//!
//! Grouped by type, then by whole-input rules. The strictness tests matter
//! more than they look: bencode has one valid spelling per value, and the
//! infohash depends on that, so accepting `i03e` would quietly break
//! round-tripping.

use super::*;
use crate::cursor::DEPTH_MAX;

// --- helpers -----------------------------------------------------------

/// Decode, expecting success.
fn ok(input: &[u8]) -> Bencode<'_> {
    decode(input).expect("expected this to decode")
}

/// Decode, expecting any failure.
///
/// Use [`err_is`] where the specific reason matters — this only proves
/// something went wrong, not that the right rule caught it.
fn err(input: &[u8]) {
    assert!(decode(input).is_err(), "expected an error for {input:?}");
}

/// Decode, expecting a specific error.
///
/// Guards against a test passing for the wrong reason: without this,
/// a decoder that rejected `i03e` as "unexpected end of input" would look
/// just as correct as one that rejected it for the leading zero.
fn err_is(input: &[u8], expected: &DecoderError) {
    match decode(input) {
        Err(actual) => assert_eq!(
            std::mem::discriminant(&actual),
            std::mem::discriminant(expected),
            "wrong error for {input:?}: got {actual:?}, wanted {expected:?}"
        ),
        Ok(value) => panic!("expected an error for {input:?}, got {value:?}"),
    }
}

fn as_int(value: &Bencode) -> i64 {
    match value {
        Bencode::Integer(number) => *number,
        other => panic!("expected an integer, got {other:?}"),
    }
}

fn as_str<'a>(value: &Bencode<'a>) -> &'a [u8] {
    match value {
        Bencode::String(bytes) => bytes,
        other => panic!("expected a string, got {other:?}"),
    }
}

// --- integers ----------------------------------------------------------

#[test]
fn integer_positive() {
    assert_eq!(as_int(&ok(b"i42e")), 42);
}

#[test]
fn integer_negative() {
    assert_eq!(as_int(&ok(b"i-42e")), -42);
}

/// A bare zero is legal — only zeros *followed by digits* are not.
#[test]
fn integer_zero_is_legal() {
    assert_eq!(as_int(&ok(b"i0e")), 0);
}

#[test]
fn integer_rejects_leading_zero() {
    err_is(b"i03e", &DecoderError::LeadingZero);
    err_is(b"i007e", &DecoderError::LeadingZero);
}

#[test]
fn integer_rejects_negative_zero() {
    err_is(b"i-0e", &DecoderError::NegativeZero);
}

#[test]
fn integer_rejects_empty() {
    err_is(b"ie", &DecoderError::EmptyNumber);
    err_is(b"i-e", &DecoderError::EmptyNumber);
}

#[test]
fn integer_rejects_unterminated() {
    err(b"i42");
}

/// A misplaced sign must not be read as a digit.
#[test]
fn integer_rejects_sign_in_middle() {
    err(b"i1-2e");
}

/// A hostile file can claim a number too large to hold. It must error
/// rather than wrap silently into a small one.
#[test]
fn integer_rejects_overflow() {
    err_is(
        b"i99999999999999999999999999e",
        &DecoderError::NumberOverflow,
    );
}

// --- strings -----------------------------------------------------------

#[test]
fn string_normal() {
    assert_eq!(as_str(&ok(b"4:spam")), b"spam");
}

#[test]
fn string_empty_is_legal() {
    assert_eq!(as_str(&ok(b"0:")), b"");
}

#[test]
fn string_rejects_short_data() {
    err(b"5:abc");
}

#[test]
fn string_rejects_leading_zero_length() {
    err_is(b"04:spam", &DecoderError::LeadingZero);
}

/// A length far beyond the buffer must fail on its own terms, not by
/// attempting an enormous allocation.
#[test]
fn string_rejects_absurd_length() {
    err(b"99999999999999999999:x");
}

/// Byte strings are not text — filenames and SHA-1 hashes are not UTF-8.
#[test]
fn string_holds_arbitrary_bytes() {
    let input = b"3:\xff\xfe\xfd";
    assert_eq!(as_str(&ok(input)), &[0xff, 0xfe, 0xfd]);
}

// --- lists -------------------------------------------------------------

#[test]
fn list_empty() {
    match ok(b"le") {
        Bencode::List(items) => assert!(items.is_empty()),
        other => panic!("expected a list, got {other:?}"),
    }
}

#[test]
fn list_with_items() {
    match ok(b"l4:spami42ee") {
        Bencode::List(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(as_str(&items[0]), b"spam");
            assert_eq!(as_int(&items[1]), 42);
        }
        other => panic!("expected a list, got {other:?}"),
    }
}

#[test]
fn list_rejects_unterminated() {
    err(b"l4:spam");
}

// --- dictionaries ------------------------------------------------------

#[test]
fn dictionary_empty() {
    match ok(b"de") {
        Bencode::Dictionary(entries) => assert!(entries.is_empty()),
        other => panic!("expected a dictionary, got {other:?}"),
    }
}

/// The key/value desync test — if a value is parsed using the key's leading
/// byte, the second pair lands in the wrong place and this catches it.
#[test]
fn dictionary_with_pairs() {
    match ok(b"d3:cow3:moo4:spam4:eggse") {
        Bencode::Dictionary(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(as_str(&entries[b"cow".as_slice()]), b"moo");
            assert_eq!(as_str(&entries[b"spam".as_slice()]), b"eggs");
        }
        other => panic!("expected a dictionary, got {other:?}"),
    }
}

#[test]
fn dictionary_rejects_non_string_key() {
    err(b"di42e3:fooe");
}

#[test]
fn dictionary_rejects_missing_value() {
    err(b"d3:cowe");
}

// --- nesting -----------------------------------------------------------

#[test]
fn nested_structures() {
    match ok(b"d4:listl1:a1:bee") {
        Bencode::Dictionary(entries) => match &entries[b"list".as_slice()] {
            Bencode::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(as_str(&items[0]), b"a");
                assert_eq!(as_str(&items[1]), b"b");
            }
            other => panic!("expected a list, got {other:?}"),
        },
        other => panic!("expected a dictionary, got {other:?}"),
    }
}

/// The depth guard must not fire on ordinary nesting.
///
/// Without this, a limit accidentally set to 2 would still pass every other
/// test in the file.
#[test]
fn accepts_reasonable_nesting() {
    // Ten nested lists, properly closed.
    let depth = 10;
    let mut input = vec![b'l'; depth];
    input.extend(std::iter::repeat(b'e').take(depth));

    assert!(decode(&input).is_ok());
}

/// Deep nesting must produce an error rather than a stack overflow, which
/// cannot be caught and would take the whole process down.
#[test]
fn rejects_excessive_nesting() {
    // Derived from the limit so raising it doesn't silently disable this
    // test — a hardcoded count would start failing for the wrong reason.
    let input = vec![b'l'; usize::from(DEPTH_MAX) + 1];

    err_is(&input, &DecoderError::CursorError(CursorError::TooDeep));
}

// --- whole-input rules -------------------------------------------------

#[test]
fn rejects_trailing_data() {
    err_is(b"i42egarbage", &DecoderError::TrailingData);
    err_is(b"4:spamX", &DecoderError::TrailingData);
}

#[test]
fn rejects_empty_input() {
    err(b"");
}

#[test]
fn rejects_unknown_type_marker() {
    err_is(b"x", &DecoderError::UnknownType('x'));
}

// --- real file ---------------------------------------------------------

/// Skips when no fixture is present, so a fresh clone still passes.
/// See the README for where to put one.
#[test]
fn decodes_a_real_torrent() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");

    let Ok(data) = std::fs::read(path) else {
        eprintln!("skipping: no torrent file at {path}");
        return;
    };

    assert!(decode(&data).is_ok());
}
