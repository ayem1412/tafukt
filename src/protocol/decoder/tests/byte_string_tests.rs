/* use crate::protocol::Bencode;
use crate::protocol::decoder::Decoder;

#[test]
fn test_parse_byte_string() {
    use std::io::BufReader;

    let bencode = "4:spam";
    let mut reader = BufReader::new(bencode.as_bytes());
    let mut deserializer = Decoder::new(&mut reader);
    let result = deserializer.parse().unwrap();

    assert_eq!(result, Bencode::String("spam".as_bytes().to_vec()))
} */
