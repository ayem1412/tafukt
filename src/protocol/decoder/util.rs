use std::str::FromStr;

use crate::protocol::decoder::error::DecoderError;

fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes.iter().map(|&byte| byte as char).collect::<String>()
}

pub fn bytes_to_integer<T: FromStr>(bytes: Vec<u8>) -> Result<T, DecoderError> {
    bytes_to_string(bytes).parse::<T>().map_err(|_| DecoderError::InvalidIntegerSyntax)
}

pub fn validate_utf8_string(bytes: &[u8]) -> Result<String, DecoderError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| DecoderError::StringInvalidUtf8)
}
