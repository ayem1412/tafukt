use bencode::{decoder, encoder};

/// decode → encode must reproduce the input exactly.
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
}

#[test]
fn round_trip_strings() {
    round_trip(b"0:");
    round_trip(b"4:spam");
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

#[test]
fn round_trip_deeply_nested() {
    round_trip(b"d1:ad1:bd1:cli1ei2eeeee");
}

/// The one that matters: a real torrent must survive untouched.
/// If this fails, your infohash will be wrong.
#[test]
fn round_trip_real_torrent() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");

    let Ok(data) = std::fs::read(path) else {
        eprintln!("skipping: no torrent at {path}");
        return;
    };

    let value = decoder::decode(&data).expect("real torrent should decode");
    let output = encoder::encode(&value);

    assert_eq!(output.len(), data.len(), "length changed");
    assert_eq!(output, data, "bytes changed");
}
