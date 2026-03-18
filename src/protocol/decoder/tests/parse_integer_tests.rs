use crate::protocol::Bencode;
use crate::protocol::decoder::error::DecoderError;
use crate::protocol::decoder::tests::decode;

#[test]
fn test_decode_zero() {
    assert!(matches!(decode("i0e"), Ok(Bencode::Integer(0))));
}

#[test]
fn test_decode_negative_zero() {
    assert!(matches!(decode("i-0e"), Err(DecoderError::IntegerNegativeZero)));
}

#[test]
fn test_decode_positive_number() {
    assert!(matches!(decode("i69e"), Ok(Bencode::Integer(69))));
}

#[test]
fn test_decode_negative_number() {
    assert!(matches!(decode("i-420e"), Ok(Bencode::Integer(-420))));
}

#[test]
fn test_decode_leading_zero() {
    assert!(matches!(decode("i010e"), Err(DecoderError::IntegerLeadingZero)));
}

#[test]
fn test_decode_no_e_terminator() {
    assert!(matches!(decode("i10"), Err(DecoderError::MissingTerminator)));
}
