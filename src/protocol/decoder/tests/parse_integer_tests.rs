use crate::protocol::Bencode;
use crate::protocol::decoder::Decoder;

#[test]
fn test_decode_zero() {
    let bencode = "i0e";
    let mut bytes = bencode.bytes().into_iter();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode();

    assert_eq!(result.is_ok(), true);
    match result.ok().unwrap() {
        Bencode::Integer(num) => assert_eq!(num, 0),
        _ => panic!("not supposed to reach here!"),
    }
}

#[test]
fn test_decode_negative_zero() {
    let bencode = "i-0e";
    let mut bytes = bencode.bytes().into_iter();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode();

    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    assert_eq!(err.to_string(), "integer is negative zero")
}

#[test]
fn test_decode_positive_number() {
    let bencode = "i69e";
    let mut bytes = bencode.bytes().into_iter();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode();

    assert_eq!(result.is_ok(), true);
    match result.ok().unwrap() {
        Bencode::Integer(num) => assert_eq!(num, 69),
        _ => panic!("not supposed to reach here!"),
    }
}

#[test]
fn test_decode_negative_number() {
    let bencode = "i-420e";
    let mut bytes = bencode.bytes().into_iter();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode();

    assert_eq!(result.is_ok(), true);
    match result.ok().unwrap() {
        Bencode::Integer(num) => assert_eq!(num, -420),
        _ => panic!("not supposed to reach here!"),
    }
}

#[test]
fn test_decode_leading_zero() {
    let bencode = "i010e";
    let mut bytes = bencode.bytes().into_iter();
    let mut decoder = Decoder::new(&mut bytes);
    let result = decoder.decode();

    assert_eq!(result.is_err(), true);

    let err = result.err().unwrap();
    assert_eq!(err.to_string(), "integer is leading with zeros")
}
