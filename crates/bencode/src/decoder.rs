use std::collections::BTreeMap;

use crate::bencode_value::BencodeValue;
use crate::cursor::{Cursor, CursorError};

pub fn decode(data: &[u8]) -> Result<BencodeValue<'_>, CursorError> {
    let mut cursor = Cursor::new(data);
    let value = parse(&mut cursor)?;

    Ok(value)
}

fn parse<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, CursorError> {
    match cursor.peek()? {
        b'0'..=b'9' => Ok(BencodeValue::String(decode_string(cursor)?)),
        b'i' => decode_integer(cursor),
        b'd' => decode_dictionary(cursor),
        b'l' => decode_list(cursor),
        _ => unimplemented!(),
    }
}

fn decode_digits(cursor: &mut Cursor, stop: u8) -> Result<usize, CursorError> {
    let mut number: usize = 0;
    // let mut seen_digit = false;

    loop {
        let byte = cursor.bump()?;
        if byte == stop {
            break;
        }

        let digit = match byte {
            b'0'..=b'9' => (byte - b'0') as usize,
            got => unimplemented!("{got}"),
        };

        number = number
            .checked_mul(10)
            .and_then(|n| n.checked_add(digit))
            .ok_or(CursorError::Overflow)?;

        // seen_digit = true;
    }

    Ok(number)
}

fn decode_string<'a>(cursor: &mut Cursor<'a>) -> Result<&'a [u8], CursorError> {
    let length = decode_digits(cursor, b':')?;

    cursor.take(length)
}

fn decode_integer<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, CursorError> {
    cursor.expect(b'i')?;

    let negative = cursor.peek()? == b'-';
    if negative {
        cursor.bump()?;
    }

    let magnitude = decode_digits(cursor, b'e')?;

    let value = i64::try_from(magnitude).map_err(|_| CursorError::Overflow)?;

    Ok(BencodeValue::Integer(if negative { -value } else { value }))
}

fn decode_dictionary<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, CursorError> {
    cursor.expect(b'd')?;

    let mut dictionary = BTreeMap::new();

    while cursor.peek()? != b'e' {
        let key = decode_string(cursor)?;
        let value = parse(cursor)?;

        dictionary.insert(key, value);
    }

    cursor.bump()?;

    Ok(BencodeValue::Dictionary(dictionary))
}

fn decode_list<'a>(cursor: &mut Cursor<'a>) -> Result<BencodeValue<'a>, CursorError> {
    cursor.expect(b'l')?;

    let mut list = Vec::new();

    while cursor.peek()? != b'e' {
        let value = parse(cursor)?;
        list.push(value);
    }

    cursor.bump()?;

    Ok(BencodeValue::List(list))
}
