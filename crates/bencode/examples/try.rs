use std::{fs, time::Instant};

use bencode::decoder;

fn main() {
    let data = fs::read(format!(
        "{}/torrents/debian.iso.torrent",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let value = decoder::decode(&data).unwrap();
        std::hint::black_box(&value);
    }
    println!("{:?} per decode", start.elapsed() / iterations);
}
