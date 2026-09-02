//! Decoding bencode into [`Bencode`] values.
//!
//! Bencode is the data format used by `.torrent` files and the `BitTorrent`
//! DHT. It has four types: integers, byte strings, lists, and dictionaries.
//!
//! # Example
//!
//! ```
//! use bencode::decoder::decode;
//!
//! let value = decode(b"d3:cow3:mooe")?;
//! # Ok::<(), bencode::decoder::DecoderError>(())
//! ```
//!
//! # Hostile input
//!
//! Every function here is safe to point at bytes from a stranger. Malformed
//! input returns an error; nothing panics, and nesting is depth-limited so a
//! deeply nested value cannot overflow the stack.
//!
//! # Strictness
//!
//! Decoding is strict, because torrent identity depends on exact bytes. Values
//! that some decoders tolerate are rejected here: leading zeros (`i03e`),
//! negative zero (`i-0e`), empty numbers (`ie`), and trailing data after the
//! top-level value. This guarantees that decoding and re-encoding reproduces
//! the input byte for byte.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::{
    bencode::Bencode,
    cursor::{Cursor, CursorError},
};

/// Something went wrong while decoding bencode.
#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    /// The top-level value ended, but bytes remained after it.
    #[error("unexpected trailing data")]
    TrailingData,

    /// A value started with a byte that begins no bencode type.
    #[error("unknown `Bencode` type {0}")]
    UnknownType(char),

    /// A number had no digits at all, as in `ie` or `0:`'s missing length.
    #[error("empty number")]
    EmptyNumber,

    /// A non-digit byte appeared where a digit was expected.
    #[error("expected a number, got: {got}")]
    UnexpectedNumber { got: u8 },

    /// A number began with `0` and continued, as in `i03e`.
    ///
    /// Bencode allows a bare `0` but no other leading zeros, since the same
    /// value must have exactly one representation.
    #[error("leading zeros are not allowed in `Bencode`")]
    LeadingZero,

    /// The number `i-0e`, which bencode does not allow.
    #[error("negative zeros are not allowed in `Bencode`")]
    NegativeZero,

    /// A number was too large to represent.
    #[error("number too large")]
    NumberOverflow,

    /// The input ended early, was nested too deeply, or a marker byte was
    /// not what the format required.
    #[error(transparent)]
    CursorError(#[from] CursorError),
}

/// Decode a single bencode value from `data`.
///
/// The returned value borrows from `data`, so no byte strings are copied.
///
/// # Errors
///
/// Returns an error if the bytes are not valid bencode, if anything remains
/// after the value, or if nesting exceeds the depth limit.
///
/// # Example
///
/// ```
/// use bencode::{bencode::Bencode, decoder::decode};
///
/// let value = decode(b"i42e")?;
/// assert_eq!(value.as_i64()?, 42);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode(data: &[u8]) -> Result<Bencode<'_>, DecoderError> {
    let mut cursor = Cursor::new(data);
    let value = parse(&mut cursor)?;

    if !cursor.is_empty() {
        return Err(DecoderError::TrailingData);
    }

    Ok(value)
}

/// Decode one value, dispatching on its first byte.
///
/// Each `decode_*` function consumes its own marker byte, so this only peeks.
fn parse<'a>(cursor: &mut Cursor<'a>) -> Result<Bencode<'a>, DecoderError> {
    match cursor.peek()? {
        b'0'..=b'9' => Ok(Bencode::String(decode_string(cursor)?)),
        b'i' => decode_integer(cursor),
        b'd' => decode_dictionary(cursor),
        b'l' => decode_list(cursor),
        got => Err(DecoderError::UnknownType(got as char)),
    }
}

/// Read ASCII digits until `stop`, accumulating them into a number.
///
/// Shared by strings (which stop at `:`) and integers (which stop at `e`).
/// The number is built digit by digit rather than through an intermediate
/// string, so no allocation happens.
///
/// Rejects an empty run of digits and any leading zero followed by more
/// digits. A bare `0` is allowed.
fn decode_digits(cursor: &mut Cursor<'_>, stop: u8) -> Result<usize, DecoderError> {
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
            .ok_or(DecoderError::NumberOverflow)?;

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

/// Decode a byte string: a length in ASCII digits, a `:`, then that many bytes.
///
/// Returns a borrow of the input rather than a copy. Byte strings are not
/// text — they hold filenames in unknown encodings and raw SHA-1 hashes — so
/// they are never converted to `String` here.
fn decode_string<'a>(cursor: &mut Cursor<'a>) -> Result<&'a [u8], DecoderError> {
    let length = decode_digits(cursor, b':')?;

    Ok(cursor.take(length)?)
}

/// Decode an integer: `i`, an optional `-`, digits, then `e`.
fn decode_integer<'a>(cursor: &mut Cursor<'a>) -> Result<Bencode<'a>, DecoderError> {
    cursor.expect(b'i')?;

    let negative = cursor.peek()? == b'-';
    if negative {
        cursor.bump()?;
    }

    let magnitude = decode_digits(cursor, b'e')?;

    if negative && magnitude == 0 {
        return Err(DecoderError::NegativeZero);
    }

    let value = i64::try_from(magnitude).map_err(|_| DecoderError::NumberOverflow)?;

    Ok(Bencode::Integer(if negative { -value } else { value }))
}

/// Decode a dictionary: `d`, then alternating keys and values, then `e`.
///
/// Keys are always byte strings. They are stored in a [`BTreeMap`], which
/// keeps them in the sorted order bencode requires — so re-encoding a decoded
/// dictionary reproduces the original byte order.
fn decode_dictionary<'a>(cursor: &mut Cursor<'a>) -> Result<Bencode<'a>, DecoderError> {
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

    Ok(Bencode::Dictionary(dictionary))
}

/// Decode a list: `l`, then any number of values, then `e`.
fn decode_list<'a>(cursor: &mut Cursor<'a>) -> Result<Bencode<'a>, DecoderError> {
    cursor.expect(b'l')?;
    cursor.enter()?;

    let mut list = vec![];

    while cursor.peek()? != b'e' {
        let value = parse(cursor)?;
        list.push(value);
    }

    cursor.bump()?;
    cursor.leave();

    Ok(Bencode::List(list))
}

/// A half-open byte range into the decoded input: `[start, end)`.
///
/// Slice the original buffer with it to recover a value's exact bytes.
pub type Span = (usize, usize);

/// A decoded top-level dictionary, with the byte range of each value.
pub struct DecodedRoot<'a> {
    /// The dictionary's keys and decoded values.
    pub root: BTreeMap<&'a [u8], Bencode<'a>>,

    /// Where each value's raw bytes sit in the input, keyed the same way.
    ///
    /// Spans cover the value only, never its key.
    pub spans: BTreeMap<&'a [u8], Span>,
}

/// Decode a top-level dictionary, recording where each value's bytes live.
///
/// Use this instead of [`decode`] when you need a value's original bytes
/// rather than its decoded form. The `BitTorrent` infohash is the motivating
/// case: it is a hash of the raw `info` dictionary exactly as written, and
/// re-encoding a decoded value can differ by a byte and produce a hash no
/// peer will accept.
///
/// Spans are recorded for direct children of the root only, which is all the
/// `.torrent` format needs.
///
/// # Errors
///
/// Returns an error if the input is not a single well-formed dictionary, or
/// if anything follows it.
///
/// # Example
///
/// ```no_run
/// use bencode::decoder::decode_dictionary_with_spans;
///
/// let data = std::fs::read("example.torrent")?;
/// let decoded = decode_dictionary_with_spans(&data)?;
///
/// let &(start, end) = decoded.spans.get(b"info".as_slice()).unwrap();
/// let info_bytes = &data[start..end]; // hash these
///     
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn decode_dictionary_with_spans(data: &[u8]) -> Result<DecodedRoot<'_>, DecoderError> {
    let mut cursor = Cursor::new(data);

    cursor.expect(b'd')?;
    cursor.enter()?;

    let mut root = BTreeMap::new();
    let mut spans = BTreeMap::new();

    while cursor.peek()? != b'e' {
        let key = decode_string(&mut cursor)?;

        // The span starts after the key, so it covers the value alone.
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
