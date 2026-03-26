use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};

use reqwest::Url;
use sha1::{Digest, Sha1};

use crate::metainfo::Metainfo;
use crate::protocol::decoder::Decoder;
use crate::protocol::{Bencode, encoder};
use crate::tracker::Tracker;

mod metainfo;
mod protocol;
mod tracker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* let invalid_string = unsafe {
        // archlinux-2026.03.01-x86_64.iso.torrent
        // 716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent
        // debian.iso.torrent
        String::from_utf8_unchecked(include_bytes!("../torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").to_vec())
    }; */

    /* let file = File::open("./torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").unwrap();
    let reader = BufReader::new(file);
    let mut bytes = reader.bytes().map(|c| c.unwrap()); */
    let file_content = std::fs::read("./torrents/debian.iso.torrent").unwrap();
    let mut bytes = file_content.iter().copied();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode().unwrap();

    // println!("{result}");
    let metainfo = Metainfo::try_from(result).unwrap();
    // println!("{}", Into::<Bencode>::into(metainfo.info));
    // println!("{:#?}", metainfo.info.);

    let length = metainfo.info.length();
    let info_hash = metainfo.info.info_hash().unwrap();
    // let hex = info_hash.into_iter().map(|b| format!("%{:02X}", b)).collect::<String>();
    let tracker = Tracker::new("-PC0001-123456789012".into(), 6881, 0, 0, length, 1, metainfo.announce, info_hash);
    let result = tracker.get().await?;
    println!("{:#?}", result);

    Ok(())
}
