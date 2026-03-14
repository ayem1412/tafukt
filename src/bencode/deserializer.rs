use std::{char, io, slice};

use crate::bencode::Bencode;
use crate::util;

pub struct Deserializer<'a, R: io::Read>(&'a mut R);

impl<'a, R: io::Read> Deserializer<'a, R> {
    pub fn new(reader: &'a mut R) -> Self {
        Self(reader)
    }

    /// Reads each byte and parses it into a valid Bencode.
    fn parse(&mut self) -> io::Result<Bencode> {
        let byte = self.read_byte()?;
        match byte {
            b'i' => self.parse_integer(),
            b'0'..=b'9' => Ok(Bencode::String(self.parse_byte_string(byte)?)),
            _ => panic!("aa"),
        }
    }

    // Parses an integer into a valid Bencode.
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
        let mut buffer: Vec<u8> = Vec::with_capacity(string_length);
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

            byte = self.read_byte()?;
            if byte.eq(&b':') {
                break;
            }

            string_length.push(char::from(head));
        }

        if string_length.is_empty() {
            util::invalid_data_error(format!("Invalid integer syntax!, Received: {}", string_length).as_str())?
        }

        string_length.parse::<usize>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Integer overflow!, Received: {}", string_length).as_str(),
            )
        })
    }

    /// Reads from the `Reader` exactly one byte and return it.
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut byte = 0u8;
        self.0.read_exact(slice::from_mut(&mut byte))?;
        Ok(byte)
    }
}
