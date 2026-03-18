use crate::protocol::Bencode;
use crate::protocol::decoder::tests::decode;

#[test]
fn test_decode_list_string() {
    let list = vec![Bencode::String(b"spam".to_vec()), Bencode::String(b"eggs".to_vec())];
    assert_eq!(decode("l4:spam4:eggse"), Ok(Bencode::List(list)))
}

#[test]
fn test_decode_list_integer() {
    let list = vec![Bencode::Integer(4), Bencode::Integer(5)];
    assert_eq!(decode("li4ei5ee"), Ok(Bencode::List(list)))
}

#[test]
fn test_decode_list_mixed() {
    let list = vec![Bencode::Integer(4), Bencode::String(b"spam".to_vec())];
    assert_eq!(decode("li4e4:spame"), Ok(Bencode::List(list)))
}
