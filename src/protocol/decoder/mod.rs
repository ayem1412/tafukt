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
}

/*


use std::collections::BTreeMap;
use std::{char, io, slice};

use crate::protocol::Bencode;
use crate::util;

pub struct Decoder<'a, R: io::Read>(&'a mut R);

// CREDITS: This code was mostly inspired by https://www.nayuki.io/res/bittorrent-bencode-format-tools/bencode.rs
impl<'a, R: io::Read> Decoder<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Self(reader)
    }

    /// Reads from the `Reader` exactly one byte and return it.
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut byte = 0u8;
        self.0.read_exact(slice::from_mut(&mut byte))?;
        Ok(byte)
    }

    /// Reads each byte and parses it into a valid Bencode.
    fn parse(&mut self) -> io::Result<Bencode> {
        let mut byte = self.read_byte()?;
        let result = self.parse_value(byte)?;

        if self.0.read(std::slice::from_mut(&mut byte))? > 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpcted extra data!"));
        }

        Ok(result)
    }

    fn parse_value(&mut self, head: u8) -> io::Result<Bencode> {
        match head {
            b'i' => self.parse_integer(),
            b'0'..=b'9' => Ok(Bencode::String(self.parse_byte_string(head)?)),
            b'l' => self.parse_list(),
            _ => panic!("Invalid bencode: {}", head as char),
        }
    }

    /// Parses an integer into a valid Bencode.
    fn parse_integer(&mut self) -> io::Result<Bencode> {
        let mut integer_str = String::new();

        loop {
            let byte = self.read_byte()?;
            if byte.eq(&b'e') {
                break;
            }

            let valid = if integer_str.is_empty() {
                byte.eq(&b'-') || byte.ge(&b'0') && byte.le(&b'9')
            } else {
                byte.ge(&b'0') && byte.le(&b'9')
            };

            if !valid {
                util::invalid_data_error("Invalid integer syntax!")?
            }

            integer_str.push(char::from(byte));
        }

        let zero_padding_regex = regex::Regex::new(r"^0[0-9]").unwrap();
        if integer_str.is_empty() || integer_str.eq("-0") || zero_padding_regex.is_match(integer_str.as_str()) {
            util::invalid_data_error(format!("Invalid integer syntax!, Received: {}", integer_str).as_str())?
        }

        integer_str.parse::<i128>().map(Bencode::Integer).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Integer overflow!, Received: {}", integer_str).as_str())
        })
    }

    /// Parses a String into a vector of bytes.
    fn parse_byte_string(&mut self, head: u8) -> io::Result<Vec<u8>> {
        let string_length = self.parse_string_length_integer(head)?;
        let mut buffer = vec![0u8; string_length];
        self.0.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    /// Parses the length of a string.
    fn parse_string_length_integer(&mut self, head: u8) -> io::Result<usize> {
        let mut string_length = String::new();
        let mut byte = head;

        loop {
            if byte.lt(&b'1') || byte.gt(&b'9') || string_length.eq("0") {
                util::invalid_data_error("Invalid integer!")?
            }

            string_length.push(char::from(head));

            byte = self.read_byte()?;
            if byte.eq(&b':') {
                break;
            }
        }

        if string_length.is_empty() {
            util::invalid_data_error("Invalid integer syntax!")?
        }

        string_length.parse::<usize>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Integer overflow!, Received: {}", string_length).as_str(),
            )
        })
    }

    /// Parses a list into a valid Bencode.
    fn parse_list(&mut self) -> io::Result<Bencode> {
        let mut list = Vec::new();

        loop {
            match self.read_byte()? {
                b'e' => break,
                byte => list.push(self.parse_value(byte)?),
            }
        }

        Ok(Bencode::List(list))
    }

    // Parses a list into a valid Bencode.
    /* fn parse_dictionnary(&mut self) -> io::Result<Bencode> {
        let mut dictionnary = BTreeMap::new();

        loop {
            let key = match self.read_byte()? {
                b'e' => break,
                byte => self.parse_byte_string(byte)?,
            };

            let prev_key = dictionnary.keys().next_back();
            if prev_key.map_or(false, |k| key <= *k) {
            };
        }

        Ok(Bencode::Dictionnary(dictionnary))
    } */
} */
