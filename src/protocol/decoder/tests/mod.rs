use crate::protocol::decoder::{Decoder, DecoderResult};

mod list_tests;
mod parse_byte_string_tests;
mod parse_integer_tests;

pub fn decode(input: &str) -> DecoderResult {
    let mut bytes = input.as_bytes().iter().copied();
    Decoder::new(&mut bytes).decode()
}
