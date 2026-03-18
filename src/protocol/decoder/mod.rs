/**
 * CREDITS
 *
 * THIS CODE IS HEAVILY INSPIRED BY THIS AMAZING IMPLEMENTATION OF A BENCODE DECODER:
 * https://github.com/denis-selimovic/bencode/blob/main/src/protocol/decode.rs
 *
 */

#[cfg(test)]
mod tests;

use crate::protocol::Bencode;
use crate::protocol::decoder::error::DecoderError;

mod error;
mod util;

pub struct Decoder<'a, B: Iterator<Item = u8>>(pub &'a mut B);

type DecoderResult = Result<Bencode, DecoderError>;

impl<'a, B: Iterator<Item = u8>> Decoder<'a, B> {
    pub fn new(bytes: &'a mut B) -> Self {
        Self(bytes)
    }

    pub fn decode(&mut self) -> DecoderResult {
        let head = self.0.next().ok_or(DecoderError::Empty)?;
        let bencode = self.parse(head)?;

        if self.0.next().is_some() {
            return Err(DecoderError::UnexpectedExtraData);
        }

        Ok(bencode)
    }

    fn parse(&mut self, head: u8) -> DecoderResult {
        match head {
            b'i' => self.parse_integer(),
            b'0'..=b'9' => {
                let len = self.parse_string_length(head)?;
                self.parse_byte_string(len)
            },
            b'l' => self.parse_list(),
            _ => todo!(),
        }
    }

    fn parse_integer(&mut self) -> DecoderResult {
        let mut is_negative = false;
        let mut buff: Vec<u8> = vec![];

        let head = self.0.next().ok_or(DecoderError::InvalidIntegerSyntax)?;
        match head {
            b'-' => is_negative = true,
            b'0'..=b'9' => buff.push(head),
            _ => return Err(DecoderError::InvalidIntegerSyntax),
        }

        let mut found_terminator = false;
        while let Some(byte) = self.0.next() {
            match byte {
                b'e' => {
                    found_terminator = true;
                    break;
                },
                b'0'..=b'9' => buff.push(byte),
                _ => return Err(DecoderError::InvalidByte(byte)),
            }
        }

        if !found_terminator {
            return Err(DecoderError::MissingTerminator);
        }

        if buff.is_empty() {
            return Err(DecoderError::InvalidIntegerSyntax);
        }

        if buff.len() > 1 && buff[0] == b'0' {
            return Err(DecoderError::IntegerLeadingZero);
        }

        let mut integer: i64 = util::bytes_to_integer(buff)?;

        if is_negative {
            if integer == 0 {
                return Err(DecoderError::IntegerNegativeZero);
            }

            integer = integer.wrapping_neg();
        }

        Ok(Bencode::Integer(integer))
    }

    fn parse_byte_string(&mut self, len: usize) -> DecoderResult {
        let bytes = self.0.by_ref().take(len).collect::<Vec<u8>>();
        if bytes.len() != len {
            return Err(DecoderError::StringInvalidLength(len));
        }

        Ok(Bencode::String(bytes))
    }

    fn parse_string_length(&mut self, head: u8) -> Result<usize, DecoderError> {
        let mut length = vec![head];

        while let Some(byte) = self.0.next() {
            match byte {
                b':' => break,
                b'0'..=b'9' => length.push(byte),
                _ => return Err(DecoderError::InvalidByte(byte)),
            }
        }

        util::bytes_to_integer(length)
    }

    fn parse_list(&mut self) -> DecoderResult {
        let mut list = vec![];

        loop {
            let byte = self.0.next().ok_or(DecoderError::MissingTerminator)?;
            if byte == b'e' {
                break;
            }

            list.push(self.parse(byte)?);
        }

        Ok(Bencode::List(list))
    }
}
