use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};

use reqwest::Url;
use sha1::{Digest, Sha1};

use crate::metainfo::Metainfo;
use crate::protocol::decoder::Decoder;
use crate::protocol::{Bencode, encoder};

mod metainfo;
mod protocol;

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
    // reqwest can't parse binary values :(((((
    // https://github.com/seanmonstar/reqwest/issues/1613
    // let hex = info_hash.into_iter().map(|b| format!("%{:02X}", b)).collect::<String>();
    let info_hash = urlencoding::encode_binary(info_hash.as_ref());
    println!("{info_hash}");
    let client = reqwest::Client::new();
    let base_url = format!("{}?info_hash={info_hash}", metainfo.announce.unwrap());
    let url = Url::parse_with_params(
        // metainfo.announce.unwrap().as_str(),
        base_url.as_str(),
        &[
            // ("info_hash", hex),
            ("peer_id", "-PC0001-123456789012".to_string()),
            ("port", "6881".to_string()),
            ("uploaded", "0".to_string()),
            ("downloaded", "0".to_string()),
            ("left", length.to_string()),
            ("compact", "1".to_string()),
        ],
    )
    .unwrap();
    println!("{url}");
    let req = client.get(url).send().await.unwrap();
    let res = req.bytes().await.unwrap();
    let mut bytes = res.into_iter();
    // println!("{:#?}", res);
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode().expect("couldn't decode");
    match result {
        Bencode::Dictionary(dict) => {
            let peers = dict.get("peers").unwrap();
            // println!("{:#?}", peers)
            match peers {
                Bencode::String(bytes) => {
                    for chunks in bytes.chunks_exact(6) {
                        let ip = format!("{}.{}.{}.{}", chunks[0], chunks[1], chunks[2], chunks[3]);
                        let port = u16::from_be_bytes([chunks[4], chunks[5]]);

                        println!("IP {ip} PORT {port}");
                        /* let ip = format!("{}:{}:{}:{}", chunks[0], chunks[1], chunks[2], chunks[3]);
                        let port = u16::from_be_bytes([chunks[4], chunks[5]]); */
                    }
                },
                _ => unimplemented!(),
            };
        },
        _ => unimplemented!(),
    };
    // println!("{}", String::from_utf16_lossy(res.text().await.unwrap()));
    // let req = reqwest::get(url).await.unwrap().text().await.unwrap();
    // println!("{req:?}");

    /* let mut hasher = Sha1::new();
    hasher.update(encoder::encode(Bencode::from(metainfo.info)));
    let result = hasher.finalize(); */
    // 716CDB3E77094135E601A83B555CBBB3EB1D9557
    // let hex = result.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join("");
    // println!("IS EQUAL {}", hex == "716CDB3E77094135E601A83B555CBBB3EB1D9557");
    // println!("{:X}", metainfo.info.info_hash());
    // let test = reqwest::get(url);

    Ok(())
}
