use std::fs;

use bittorrent::{metainfo::Metainfo, util};

fn main() {
    let data = fs::read(format!(
        "{}/torrents/flstudio.torrent",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    let metainfo = Metainfo::from_bytes(&data).unwrap();
    // println!("{:?}", util::to_hex(&metainfo.info_hash));
}
