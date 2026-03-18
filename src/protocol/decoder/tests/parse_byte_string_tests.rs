use crate::protocol::Bencode;
use crate::protocol::decoder::error::DecoderError;
use crate::protocol::decoder::tests::decode;

#[test]
fn test_decode_string() {
    assert_eq!(decode("4:spam"), Ok(Bencode::String(b"spam".to_vec())))
}

#[test]
fn test_decode_empty_string() {
    assert_eq!(decode("0:"), Ok(Bencode::String(b"".to_vec())))
}

#[test]
fn test_decode_string_with_spaces() {
    assert_eq!(decode("11:hello world"), Ok(Bencode::String(b"hello world".to_vec())))
}

#[test]
fn test_decode_mismatched_length_string() {
    assert_eq!(decode("5:spam"), Err(DecoderError::StringInvalidLength(5)))
}

#[test]
fn test_decode_missing_terminator() {
    assert_eq!(decode("5spam"), Err(DecoderError::InvalidByte(b's')))
}
