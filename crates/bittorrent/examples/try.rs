use std::{fs, time::Instant};

use bencode::decoder;
use bittorrent::metainfo::Metainfo;

fn main() {
    let data = fs::read(format!(
        "{}/torrents/debian.iso.torrent",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    Metainfo::from_bytes(&data).unwrap();
}
