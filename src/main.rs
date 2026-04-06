use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::net::SocketAddrV4;

use reqwest::Url;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::Level;

use crate::metainfo::Metainfo;
use crate::peer::Peer;
use crate::peer::message::Message;
use crate::protocol::decoder::Decoder;
use crate::protocol::{Bencode, encoder};
use crate::tracker::Tracker;

mod metainfo;
mod peer;
mod protocol;
mod tracker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
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

    let name = metainfo.info.name.clone();
    let length = metainfo.info.length();
    let piece_count = metainfo.info.piece_count();
    let info_hash = metainfo.info.info_hash().as_ref().unwrap();
    // let hex = info_hash.into_iter().map(|b| format!("%{:02X}", b)).collect::<String>();
    let tracker = Tracker::new(metainfo.announce, info_hash);
    let response = tracker.get_peers("-PC0001-123456789012".into(), 6881, 0, 0, length, 1).await?;

    for (ip, port) in response.peers {
        let peer = Peer::new(ip, port);

        match peer.connect().await {
            Ok(mut stream) => {
                let info_hash: [u8; 20] = info_hash.as_ref().try_into()?;

                match peer.handshake(&mut stream, info_hash, "-PC0001-123456789012").await {
                    Ok(()) => {
                        peer.bitfield(&mut stream, piece_count).await?;
                        peer.interested(&mut stream).await?;
                        peer.unchoke(&mut stream).await?;

                        println!("READING MESSAGES");

                        let mut current_piece = 0u32;
                        let mut current_offset = 0u32;
                        const MAX_LENGTH: u32 = 16384;

                        loop {
                            match peer.read_message(&mut stream).await {
                                Ok(Some(peer_message)) => match peer_message.id {
                                    Message::Bitfield => {
                                        println!("RECEIVED BITFIELD ({} bytes)", peer_message.payload.len());

                                        peer.request(&mut stream, 0, 0, MAX_LENGTH).await?;
                                    },
                                    Message::Piece => {
                                        println!("PIECE");
                                        let piece = peer.piece(peer_message.payload).await?;
                                        println!("RECEIVED PIECE: {:?}", piece);

                                        println!("WRITING BLOCK");

                                        piece.save_block_to_disk(length, name.as_str()).await?;

                                        println!("BLOCK WRITTEN");

                                        current_offset += MAX_LENGTH;

                                        if current_offset as u64 >= length {
                                            current_piece += 1;
                                            current_offset = 0;
                                        }

                                        println!("LENGTH: {length} CURRENT OFFSET: {current_offset}");

                                        if current_piece < piece_count as u32 {
                                            println!("CURRENT PIECE: {current_piece} PIECE COUNT: {piece_count}");
                                            peer.request(&mut stream, current_piece, current_offset, MAX_LENGTH)
                                                .await?;
                                        } else {
                                            break;
                                        }
                                    },
                                    _ => println!("RECEIVED: {:#?}", peer_message.id),
                                },
                                Ok(None) => println!("RECEIVED KEEPALIVE"),
                                Err(err) => {
                                    eprintln!("ERROR {err}");
                                    break;
                                },
                            }
                        }
                    },
                    Err(err) => eprintln!("HANDSHAKE ERROR {err}"),
                };
            },
            Err(err) => {
                eprintln!("Peer {} failed: {}", peer.addr(), err)
            },
        };
    }

    Ok(())
}
