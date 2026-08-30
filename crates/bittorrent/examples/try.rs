use std::{fs, str::FromStr};

use bittorrent::{magnet::Magnet, metainfo::Metainfo, util};

fn main() {
    let data = fs::read(format!(
        "{}/torrents/flstudio.torrent",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    // let metainfo = Metainfo::from_bytes(&data).unwrap();
    // println!("{:?}", util::to_hex(&metainfo.info_hash));
    let magnet = Magnet::from_str(
        "magnet:?xt=urn:btih:GNNHYKBSOAZWEPR4ZLX4Z6TGZJ6N4QOL&dn=STAR+WARS+Zero+Company%3A+Deluxe+Edition+%28%2B+2+DLCs%2C+MULTi12%29+%5BFitGirl+Monkey+Repack%5D&tr=udp%3A%2F%2Fopentor.net%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce",
    ).unwrap();

    println!("{magnet:?}");
}
