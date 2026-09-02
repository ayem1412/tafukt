//! The [`Bencode`] value type and its accessors.
//!
//! A decoded bencode document is a tree of these. Values borrow from the
//! bytes they were decoded from, so nothing is copied while parsing.

use std::{collections::BTreeMap, fmt};

/// Longest string shown in full by [`Display`](fmt::Display) before truncation.
const MAX_STRING_LEN: usize = 60;

/// One level of indentation in [`Display`](fmt::Display) output.
const INDENT: &str = "  ";

/// A value was not the type the caller asked for.
#[derive(Debug, thiserror::Error)]
pub enum BencodeError {
    /// An accessor was called on the wrong variant, such as [`Bencode::as_i64`]
    /// on a byte string.
    #[error("expected: {want}, got: {got}")]
    WrongType {
        /// The type the caller asked for.
        want: &'static str,
        /// The type the value actually is.
        got: &'static str,
    },
}

/// A decoded bencode value.
///
/// The lifetime ties byte strings to the buffer they were decoded from.
///
/// Note that [`Bencode::String`] holds arbitrary bytes, not text. Torrent
/// files store filenames in unknown encodings and raw SHA-1 hashes as
/// strings, neither of which is valid UTF-8.
#[derive(Debug)]
pub enum Bencode<'a> {
    /// A byte string, written as its length, a `:`, then the bytes.
    String(&'a [u8]),

    /// A signed integer, written as `i`, digits, then `e`.
    Integer(i64),

    /// A dictionary with byte-string keys, written as `d`, pairs, then `e`.
    ///
    /// Uses a [`BTreeMap`] because bencode requires keys in sorted byte
    /// order — which means re-encoding preserves the original ordering.
    Dictionary(BTreeMap<&'a [u8], Self>),

    /// A list of values, written as `l`, items, then `e`.
    List(Vec<Self>),
}

impl Bencode<'_> {
    /// The name of this value's type, for error messages.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Bencode::String(_) => "string",
            Bencode::Integer(_) => "integer",
            Bencode::Dictionary(_) => "dictionary",
            Bencode::List(_) => "list",
        }
    }

    /// Read this value as an integer.
    ///
    /// # Errors
    ///
    /// Returns [`BencodeError::WrongType`] if the value is not an integer.
    pub const fn as_i64(&self) -> Result<i64, BencodeError> {
        match self {
            Bencode::Integer(number) => Ok(*number),
            _ => Err(self.wrong_type("integer")),
        }
    }

    /// Read this value as a byte string.
    ///
    /// The bytes are not guaranteed to be text — convert them only where you
    /// know the encoding, and prefer a lossy conversion for anything a user
    /// will see.
    ///
    /// # Errors
    ///
    /// Returns [`BencodeError::WrongType`] if the value is not a byte string.
    pub const fn as_bytes(&self) -> Result<&[u8], BencodeError> {
        match self {
            Bencode::String(bytes) => Ok(bytes),
            _ => Err(self.wrong_type("string")),
        }
    }

    /// Read this value as a dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`BencodeError::WrongType`] if the value is not a dictionary.
    pub const fn as_dictionary(&self) -> Result<&BTreeMap<&[u8], Self>, BencodeError> {
        match self {
            Bencode::Dictionary(entries) => Ok(entries),
            _ => Err(self.wrong_type("dictionary")),
        }
    }

    /// Read this value as a list.
    ///
    /// # Errors
    ///
    /// Returns [`BencodeError::WrongType`] if the value is not a list.
    pub fn as_list(&self) -> Result<&[Self], BencodeError> {
        match self {
            Bencode::List(items) => Ok(items),
            _ => Err(self.wrong_type("list")),
        }
    }

    /// Build a type-mismatch error naming what was wanted and what is here.
    const fn wrong_type(&self, want: &'static str) -> BencodeError {
        BencodeError::WrongType {
            want,
            got: self.kind(),
        }
    }

    /// Write this value indented to `depth`, recursing into containers.
    fn write_at(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        match self {
            Bencode::Integer(number) => write!(f, "{number}"),

            Bencode::String(bytes) => write_bytes(f, bytes),

            Bencode::List(items) => {
                if items.is_empty() {
                    return write!(f, "[]");
                }

                writeln!(f, "[")?;
                for item in items {
                    write_indent(f, depth + 1)?;
                    item.write_at(f, depth + 1)?;
                    writeln!(f, ",")?;
                }
                write_indent(f, depth)?;
                write!(f, "]")
            }

            Bencode::Dictionary(entries) => {
                if entries.is_empty() {
                    return write!(f, "{{}}");
                }

                writeln!(f, "{{")?;
                for (key, value) in entries {
                    write_indent(f, depth + 1)?;
                    write_bytes(f, key)?;
                    write!(f, ": ")?;
                    value.write_at(f, depth + 1)?;
                    writeln!(f, ",")?;
                }
                write_indent(f, depth)?;
                write!(f, "}}")
            }
        }
    }
}

/// Write `depth` levels of indentation.
fn write_indent(f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        f.write_str(INDENT)?;
    }
    Ok(())
}

/// Write a byte string readably.
///
/// Valid UTF-8 is shown as text, truncated past [`MAX_STRING_LEN`]. Anything
/// else is summarised by length — without this, a torrent's `pieces` field
/// would print as tens of thousands of numbers.
fn write_bytes(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    match std::str::from_utf8(bytes) {
        Ok(text) if text.len() <= MAX_STRING_LEN => write!(f, "\"{text}\""),

        Ok(text) => {
            // Cut on a character boundary — slicing mid-character panics, and
            // torrent names are full of non-ASCII.
            let cut = text
                .char_indices()
                .take_while(|(i, _)| *i <= MAX_STRING_LEN)
                .last()
                .map_or(0, |(i, _)| i);

            write!(f, "\"{}…\" ({} bytes)", &text[..cut], bytes.len())
        }

        Err(_) => write!(f, "<{} bytes binary>", bytes.len()),
    }
}

/// Renders as an indented tree, for inspecting decoded data.
///
/// This is for reading, not for round-tripping — use
/// [`encode`](crate::encoder::encode) to get bytes back.
impl fmt::Display for Bencode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_at(f, 0)
    }
}
