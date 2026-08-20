use std::fs;

use bencode::decoder;

fn main() {
    let data = fs::read(format!(
        "{}/torrents/debian.iso.torrent",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    let value = decoder::decode(&data).unwrap();
    println!("{value}");
}
