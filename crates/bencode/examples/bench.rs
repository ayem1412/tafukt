//! Rough timing for the bencode decoder and encoder.
//!
//! Run with `--release`. Debug builds are an order of magnitude slower and
//! the numbers mean nothing:
//!
//! ```text
//! cargo run --release -p bencode --example bench
//! ```
//!
//! This is a sanity check, not a benchmark suite. For tracking performance
//! across changes, use `criterion`, which handles warmup and statistics
//! properly.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use bencode::{decoder, encoder};

/// Measured runs per phase.
const ITERATIONS: u32 = 10_000;

/// Unmeasured runs first, so cold caches and CPU frequency ramp-up don't
/// land in the timings.
const WARMUP: u32 = 100;

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");
    let data = std::fs::read(path).expect("put a .torrent at crates/bencode/torrents/");

    println!("file: {} bytes\n", data.len());

    // A running total forces the compiler to keep the work: without a result
    // that visibly escapes, the whole loop can be optimised away and the
    // timings become nonsense.
    let mut sink: usize = 0;

    // --- decode ------------------------------------------------------------

    for _ in 0..WARMUP {
        let value = decoder::decode(black_box(&data)).unwrap();
        sink = sink.wrapping_add(value.kind().len());
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let value = decoder::decode(black_box(&data)).unwrap();
        sink = sink.wrapping_add(value.kind().len());
    }
    let decode_time = start.elapsed() / ITERATIONS;

    // --- encode ------------------------------------------------------------

    // Decoded once, outside the timer — this phase measures encoding alone.
    let value = decoder::decode(&data).unwrap();

    for _ in 0..WARMUP {
        sink = sink.wrapping_add(encoder::encode(black_box(&value)).len());
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        sink = sink.wrapping_add(encoder::encode(black_box(&value)).len());
    }
    let encode_time = start.elapsed() / ITERATIONS;

    // --- round trip --------------------------------------------------------

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let decoded = decoder::decode(black_box(&data)).unwrap();
        sink = sink.wrapping_add(encoder::encode(&decoded).len());
    }
    let round_trip_time = start.elapsed() / ITERATIONS;

    // --- results -----------------------------------------------------------

    let megabytes = data.len() as f64 / 1_000_000.0;
    let throughput = |elapsed: Duration| megabytes / elapsed.as_secs_f64();

    println!(
        "decode:     {decode_time:>10?}  ({:.0} MB/s)",
        throughput(decode_time)
    );
    println!(
        "encode:     {encode_time:>10?}  ({:.0} MB/s)",
        throughput(encode_time)
    );
    println!(
        "round trip: {round_trip_time:>10?}  ({:.0} MB/s)",
        throughput(round_trip_time)
    );

    // The encoder must reproduce the input byte for byte — the property the
    // infohash depends on.
    let reencoded = encoder::encode(&decoder::decode(&data).unwrap());
    println!("\nround-trip identical: {}", reencoded == data);

    // Printing the total is what makes it impossible to discard.
    println!("checksum: {sink}");
}
