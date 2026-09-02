//! Print the contents of a `.torrent` file.
//!
//! ```text
//! cargo run -p bencode --example try
//! ```
//!
//! For timings, see the `bench` example.

use std::fs;

use bencode::decoder;

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/torrents/debian.iso.torrent");
    let data = fs::read(path).expect("put a .torrent at crates/bencode/torrents/");

    let value = decoder::decode(&data).expect("should be valid bencode");

    // Display truncates long strings and summarises binary, so the `pieces`
    // field doesn't flood the terminal with thousands of numbers.
    println!("{value}");
}
