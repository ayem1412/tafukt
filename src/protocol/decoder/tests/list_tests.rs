/* use crate::protocol::Bencode;
use crate::protocol::decoder::Decoder;

#[test]
fn test_parse_list() {
    use std::io::BufReader;

    let bencode = "l4:spam4:eggse";
    let mut reader = BufReader::new(bencode.as_bytes());
    let mut deserializer = Decoder::new(&mut reader);
    let result = deserializer.parse().unwrap();

    let spam = "spam".as_bytes().to_vec();
    let eggs = "eggs".as_bytes().to_vec();
    let list = Bencode::List(vec![Bencode::String(spam), Bencode::String(eggs)]);
    assert_eq!(result, list)
} */
