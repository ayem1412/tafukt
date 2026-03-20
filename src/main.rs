use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};

use sha1::{Digest, Sha1};

use crate::metainfo::Metainfo;
use crate::protocol::decoder::Decoder;

mod metainfo;
mod protocol;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* let invalid_string = unsafe {
        // archlinux-2026.03.01-x86_64.iso.torrent
        // 716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent
        String::from_utf8_unchecked(include_bytes!("../torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").to_vec())
    }; */

    let file = File::open("./torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").unwrap();
    let reader = BufReader::new(file);
    let mut bytes = reader.bytes().map(|c| c.unwrap());
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode().unwrap();
    // println!("{result}");
    let metainfo = Metainfo::try_from(result).unwrap();
    println!("METAINFO: {:?}", metainfo.announce);

    let url = metainfo.announce.unwrap();
    let hasher = Sha1::new();
    // hasher.update(metainfo.info);
    let test = reqwest::get(url);

    Ok(())
}
