//! Round-trip tests: decode then encode must reproduce the input exactly.
//!
//! These live in `tests/` rather than beside the source because they exercise
//! both halves of the crate through its public API — which also checks that
//! everything needed is actually public.
//!
//! The property matters beyond tidiness. A torrent's identity is a SHA-1 of
//! the raw bytes of its `info` dictionary. If decoding and re-encoding
//! differed by even one byte, any hash computed from parsed data would be
//! wrong, and no peer would accept the handshake.

use bencode::{decoder, encoder};

/// Decode `input`, encode it again, and require identical bytes.
fn round_trip(input: &[u8]) {
    let value = decoder::decode(input).expect("should decode");
    let output = encoder::encode(&value);

    assert_eq!(
        output,
        input,
        "round-trip changed the bytes\n  in:  {:?}\n  out: {:?}",
        String::from_utf8_lossy(input),
        String::from_utf8_lossy(&output),
    );
}

#[test]
fn round_trip_integers() {
    round_trip(b"i0e");
    round_trip(b"i42e");
    round_trip(b"i-42e");
    round_trip(b"i9223372036854775807e");
    round_trip(b"i-9223372036854775808e");
}

#[test]
fn round_trip_strings() {
    round_trip(b"0:");
    round_trip(b"4:spam");
    // Arbitrary bytes, including a null — both appear in real torrents.
    round_trip(b"3:\xff\x00\xfe");
}

#[test]
fn round_trip_lists() {
    round_trip(b"le");
    round_trip(b"l4:spami42ee");
    round_trip(b"llee");
}

#[test]
fn round_trip_dictionaries() {
    round_trip(b"de");
    round_trip(b"d3:cow3:moo4:spam4:eggse");
    round_trip(b"d4:listl1:a1:bee");
}

/// Keys already in sorted order must stay put — the encoder must not reorder
/// or renormalise anything it was given.
#[test]
fn round_trip_preserves_key_order() {
    round_trip(b"d1:ai1e2:abi2e1:bi3ee");
}

#[test]
fn round_trip_deeply_nested() {
    round_trip(b"d1:ad1:bd1:cli1ei2eeeee");
}

/// The recorded span of a value must cover exactly that value's bytes —
/// no key, no surrounding markers.
///
/// This is the property the infohash depends on, and the plain round-trip
/// above does not test it: the decoder could round-trip perfectly while
/// still reporting spans that are off by a byte.
#[test]
fn spans_cover_exactly_their_values() {
    let data = b"d3:cow3:moo4:spami42ee";

    let decoded = decoder::decode_dictionary_with_spans(data).expect("should decode");

    let (start, end) = decoded.spans[b"cow".as_slice()];
    assert_eq!(&data[start..end], b"3:moo");

    let (start, end) = decoded.spans[b"spam".as_slice()];
    assert_eq!(&data[start..end], b"i42e");
}

/// A span over a nested container must include its opening and closing
/// markers, so the slice is itself valid bencode.
#[test]
fn spans_cover_whole_containers() {
    let data = b"d4:infod3:cow3:mooee";

    let decoded = decoder::decode_dictionary_with_spans(data).expect("should decode");

    let (start, end) = decoded.spans[b"info".as_slice()];
    let info_bytes = &data[start..end];

    assert_eq!(info_bytes, b"d3:cow3:mooe");
    // The slice must stand alone as a bencode value.
    assert!(decoder::decode(info_bytes).is_ok());
}

/// The test that matters most: a real torrent must survive untouched.
///
/// Skips when no fixture is present, so a fresh clone still passes. See the
/// README for where to put one.
#[test]
fn round_trip_real_torrent() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");

    let Ok(data) = std::fs::read(path) else {
        eprintln!("skipping: no torrent at {path}");
        return;
    };

    let value = decoder::decode(&data).expect("real torrent should decode");
    let output = encoder::encode(&value);

    // Checked separately because a length mismatch says "something was
    // dropped or added", which is a far better clue than a diff of several
    // hundred thousand bytes.
    assert_eq!(output.len(), data.len(), "length changed");
    assert_eq!(output, data, "bytes changed");
}
