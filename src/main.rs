use std::fs::File;
use std::io::{BufReader, Read};

use crate::metainfo::Metainfo;
use crate::protocol::decoder::Decoder;

mod metainfo;
mod protocol;
mod util;

fn main() {
    /* let invalid_string = unsafe {
        // archlinux-2026.03.01-x86_64.iso.torrent
        // 716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent
        String::from_utf8_unchecked(include_bytes!("../torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").to_vec())
    }; */

    let file = File::open("./torrents/archlinux-2026.03.01-x86_64.iso.torrent").unwrap();
    let reader = BufReader::new(file);
    let mut bytes = reader.bytes().map(|c| c.unwrap());
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode().unwrap();
    // println!("{result}");
    let metainfo = Metainfo::try_from(result).unwrap();
    println!("METAINFO: {:#?}", metainfo.info.piece_count());
}
