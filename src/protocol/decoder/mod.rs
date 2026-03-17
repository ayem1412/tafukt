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
        let result = match self.0.next() {
            Some(head) => self.parse(head),
            None => Err(DecoderError::Empty),
        };

        if result.is_err() {
            return result;
        }

        match self.0.next() {
            Some(_) => Err(DecoderError::UnexpectedExtraData),
            None => result,
        }
    }

    fn parse(&mut self, head: u8) -> DecoderResult {
        match head {
            b'i' => self.parse_integer(),
            _ => todo!(),
        }
    }

    fn parse_integer(&mut self) -> DecoderResult {
        let mut is_negative = false;
        let mut buff: Vec<u8> = vec![];

        let head = match self.0.next() {
            Some(head) => head,
            None => return Err(DecoderError::InvalidIntegerSyntax),
        };

        if head.eq(&b'-') {
            is_negative = true;
        } else if head.ge(&b'0') && head.le(&b'9') {
            buff.push(head);
        } else {
            return Err(DecoderError::InvalidIntegerSyntax);
        }

        while let Some(byte) = self.0.next() {
            match byte {
                b'e' => break,
                b'0'..=b'9' => buff.push(byte),
                _ => return Err(DecoderError::InvalidByte("integer".to_string())),
            }
        }

        if buff.len() > 1 && buff[0] == b'0' {
            return Err(DecoderError::IntegerLeadingZero);
        }

        let mut integer = util::bytes_to_integer(buff)?;
        if integer.eq(&0) && is_negative {
            return Err(DecoderError::IntegerNegativeZero);
        }

        if is_negative {
            integer *= -1
        };

        Ok(Bencode::Integer(integer))
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
