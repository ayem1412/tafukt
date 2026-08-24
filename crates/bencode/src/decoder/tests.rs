use super::*;

// --- helpers -----------------------------------------------------------

/// Decode and expect success.
fn ok(input: &[u8]) -> Bencode<'_> {
    decode(input).expect("expected this to decode")
}

/// Decode and expect failure.
fn err(input: &[u8]) {
    assert!(decode(input).is_err(), "expected an error for {input:?}");
}

fn as_int(value: &Bencode) -> i64 {
    match value {
        Bencode::Integer(n) => *n,
        other => panic!("expected an integer, got {other:?}"),
    }
}

fn as_str<'a>(value: &Bencode<'a>) -> &'a [u8] {
    match value {
        Bencode::String(s) => s,
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

#[test]
fn integer_zero_is_legal() {
    assert_eq!(as_int(&ok(b"i0e")), 0);
}

#[test]
fn integer_rejects_leading_zero() {
    err(b"i03e");
    err(b"i007e");
}

#[test]
fn integer_rejects_negative_zero() {
    err(b"i-0e");
}

#[test]
fn integer_rejects_empty() {
    err(b"ie");
    err(b"i-e");
}

#[test]
fn integer_rejects_unterminated() {
    err(b"i42");
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
    err(b"04:spam");
}

#[test]
fn string_holds_arbitrary_bytes() {
    // Not valid UTF-8 — must still round-trip as raw bytes.
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
        Bencode::Dictionary(map) => assert!(map.is_empty()),
        other => panic!("expected a dictionary, got {other:?}"),
    }
}

/// The key/value desync test — if values are parsed with a stale head byte,
/// this is where it shows up.
#[test]
fn dictionary_with_pairs() {
    match ok(b"d3:cow3:moo4:spam4:eggse") {
        Bencode::Dictionary(map) => {
            assert_eq!(map.len(), 2);
            assert_eq!(as_str(&map[b"cow".as_slice()]), b"moo");
            assert_eq!(as_str(&map[b"spam".as_slice()]), b"eggs");
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
        Bencode::Dictionary(map) => match &map[b"list".as_slice()] {
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

#[test]
fn rejects_excessive_nesting() {
    // 200 opening list markers, never closed.
    let input = vec![b'l'; 200];
    // Must return an error, NOT overflow the stack.
    err(&input);
}

// --- whole-input rules -------------------------------------------------

#[test]
fn rejects_trailing_data() {
    err(b"i42egarbage");
    err(b"4:spamX");
}

#[test]
fn rejects_empty_input() {
    err(b"");
}

#[test]
fn rejects_unknown_type_marker() {
    err(b"x");
}

// --- real file ---------------------------------------------------------

#[test]
fn decodes_a_real_torrent() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");
    let Ok(data) = std::fs::read(path) else {
        eprintln!("skipping: no torrent file at {path}");
        return;
    };
    assert!(decode(&data).is_ok());
}
