use std::fs::File;
use std::io::BufReader;

mod bencode;
mod util;

fn main() {
    /* let invalid_string = unsafe {
        // archlinux-2026.03.01-x86_64.iso.torrent
        // 716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent
        String::from_utf8_unchecked(include_bytes!("../torrents/716CDB3E77094135E601A83B555CBBB3EB1D9557.torrent").to_vec())
    }; */

    let file = File::open("./torrents/archlinux-2026.03.01-x86_64.iso.torrent").unwrap();
    let _reader = BufReader::new(file);
    // const BYTES_TO_READ: usize = 10;
    /* let buf = 0u8;
    reader.read_exact().unwrap();
    println!("buf 1: {}", buf);
    reader.read_exact(&mut [buf]).unwrap();
    println!("buf 2: {}", buf);
    reader.read_exact(&mut [buf]).unwrap();
    println!("buf 3: {}", buf); */

    /* for byte_result in reader.bytes().take(BYTES_TO_READ) {
        match byte_result {
            Ok(byte) => print!("{}", byte as char),
            Err(_) => todo!(),
        }
    }
    println!(); */

    /* let mut deserializer = Deserializer::new(&mut reader);
    let result = deserializer.parse().unwrap(); */
}
