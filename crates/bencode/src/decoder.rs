use std::collections::BTreeMap;

use crate::bencode_value::BencodeValue;
use crate::cursor::{Cursor, CursorError};

#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    #[error("unexpected trailing data")]
    TrailingData,
    #[error("unknown `Bencode` type {0}")]
    UnknownType(char),
    #[error("empty number")]
    EmptyNumber,
    #[error("expected a number, got: {got}")]
    UnexpectedNumber { got: u8 },
    #[error("leading zeros are not allowed in `Bencode`")]
    LeadingZero,
    #[error("negative zeros are not allowed in `Bencode`")]
    NegativeZero,
    #[error("parse error")]
    CursorError(#[from] CursorError),
}

pub fn decode(data: &[u8]) -> Result<BencodeValue<'_>, DecoderError> {
    let mut cursor = Cursor::new(data);
    let value = parse(&mut cursor)?;

    if !cursor.is_empty() {
        return Err(DecoderError::TrailingData);
    }

    Ok(value)
}

fn parse<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, DecoderError> {
    match cursor.peek()? {
        b'0'..=b'9' => Ok(BencodeValue::String(decode_string(cursor)?)),
        b'i' => Ok(decode_integer(cursor)?),
        b'd' => Ok(decode_dictionary(cursor)?),
        b'l' => Ok(decode_list(cursor)?),
        got => Err(DecoderError::UnknownType(got as char)),
    }
}

fn decode_digits(cursor: &mut Cursor, stop: u8) -> Result<usize, DecoderError> {
    let mut number: usize = 0;
    let mut digits: usize = 0;
    let mut leading_zeros = false;

    loop {
        let byte = cursor.bump()?;
        if byte == stop {
            break;
        }

        let digit = match byte {
            b'0'..=b'9' => (byte - b'0') as usize,
            got => return Err(DecoderError::UnexpectedNumber { got }),
        };

        if digits == 0 && digit == 0 {
            leading_zeros = true;
        }

        number = number
            .checked_mul(10)
            .and_then(|n| n.checked_add(digit))
            .ok_or(CursorError::Overflow)?;

        digits += 1;
    }

    if digits == 0 {
        return Err(DecoderError::EmptyNumber);
    }

    if leading_zeros && digits > 1 {
        return Err(DecoderError::LeadingZero);
    }

    Ok(number)
}

fn decode_string<'a>(cursor: &mut Cursor<'a>) -> Result<&'a [u8], DecoderError> {
    let length = decode_digits(cursor, b':')?;

    Ok(cursor.take(length)?)
}

fn decode_integer<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, DecoderError> {
    cursor.expect(b'i')?;

    let negative = cursor.peek()? == b'-';
    if negative {
        cursor.bump()?;
    }

    let magnitude = decode_digits(cursor, b'e')?;

    if negative && magnitude == 0 {
        return Err(DecoderError::NegativeZero);
    }

    let value = i64::try_from(magnitude).map_err(|_| CursorError::Overflow)?;

    Ok(BencodeValue::Integer(if negative { -value } else { value }))
}

fn decode_dictionary<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, DecoderError> {
    cursor.expect(b'd')?;
    cursor.enter()?;

    let mut dictionary = BTreeMap::new();

    while cursor.peek()? != b'e' {
        let key = decode_string(cursor)?;
        let value = parse(cursor)?;

        dictionary.insert(key, value);
    }

    cursor.bump()?;
    cursor.leave();

    Ok(BencodeValue::Dictionary(dictionary))
}

fn decode_list<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, DecoderError> {
    cursor.expect(b'l')?;
    cursor.enter()?;

    let mut list = vec![];

    while cursor.peek()? != b'e' {
        let value = parse(cursor)?;
        list.push(value);
    }

    cursor.bump()?;
    cursor.leave();

    Ok(BencodeValue::List(list))
}

/// A half-open range into the original bytes: [start, end).
pub type Span = (usize, usize);

pub struct DecodedRoot<'a> {
    pub root: BTreeMap<&'a [u8], BencodeValue<'a>>,
    pub spans: BTreeMap<&'a [u8], Span>,
}

pub fn decode_dictionary_with_spans(data: &[u8]) -> Result<DecodedRoot<'_>, DecoderError> {
    let mut cursor = Cursor::new(data);

    cursor.expect(b'd')?;
    cursor.enter()?;

    let mut root = BTreeMap::new();
    let mut spans = BTreeMap::new();

    while cursor.peek()? != b'e' {
        let key = decode_string(&mut cursor)?;

        let start = cursor.pos();
        let value = parse(&mut cursor)?;
        let end = cursor.pos();

        root.insert(key, value);
        spans.insert(key, (start, end));
    }

    cursor.bump()?;

    if !cursor.is_empty() {
        return Err(DecoderError::TrailingData);
    }

    cursor.leave();

    Ok(DecodedRoot { root, spans })
}
