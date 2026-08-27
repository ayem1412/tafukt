use std::hint::black_box;
use std::time::Instant;

use bencode::{decoder, encoder};

const ITERATIONS: u32 = 10_000;

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");
    let data = std::fs::read(path).expect("put a .torrent at crates/bencode/torrents/");

    println!("file: {} bytes\n", data.len());

    // --- decode ------------------------------------------------------------

    // Warm up: first run pays for cold caches and page faults.
    for _ in 0..100 {
        black_box(decoder::decode(&data).unwrap());
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        black_box(decoder::decode(&data).unwrap());
    }
    let decode_time = start.elapsed() / ITERATIONS;

    // --- encode ------------------------------------------------------------

    // Decode once, outside the timer — we're measuring encoding only.
    let value = decoder::decode(&data).unwrap();

    for _ in 0..100 {
        black_box(encoder::encode(&value));
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        black_box(encoder::encode(&value));
    }
    let encode_time = start.elapsed() / ITERATIONS;

    // --- round trip --------------------------------------------------------

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let v = decoder::decode(&data).unwrap();
        black_box(encoder::encode(&v));
    }
    let round_trip_time = start.elapsed() / ITERATIONS;

    // --- results -----------------------------------------------------------

    let mb = data.len() as f64 / 1_000_000.0;
    let throughput = |d: std::time::Duration| mb / d.as_secs_f64();

    println!(
        "decode:     {decode_time:>10?}  ({:.0} MB/s)",
        throughput(decode_time)
    );
    println!(
        "encode:     {encode_time:>10?}  ({:.0} MB/s)",
        throughput(encode_time)
    );
    println!("round trip: {round_trip_time:>10?}");

    // Sanity: the encoder should reproduce the input exactly.
    let out = encoder::encode(&decoder::decode(&data).unwrap());
    println!("\nround-trip identical: {}", out == data);
}
