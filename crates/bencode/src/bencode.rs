use std::{collections::BTreeMap, fmt};

const MAX_STRING_LEN: usize = 60;
const INDENT: &str = "  ";

#[derive(Debug, thiserror::Error)]
pub enum BencodeError {
    #[error("expected: {want}, got: {got}")]
    WrongType {
        want: &'static str,
        got: &'static str,
    },
}

#[derive(Debug)]
pub enum Bencode<'a> {
    String(&'a [u8]),
    Integer(i64),
    Dictionary(BTreeMap<&'a [u8], Self>),
    List(Vec<Self>),
}

impl Bencode<'_> {
    pub const fn kind(&self) -> &'static str {
        match self {
            Bencode::String(_) => "string",
            Bencode::Integer(_) => "integer",
            Bencode::Dictionary(_) => "dictionary",
            Bencode::List(_) => "list",
        }
    }

    pub const fn as_i64(&self) -> Result<i64, BencodeError> {
        match self {
            Bencode::Integer(number) => Ok(*number),
            _ => Err(BencodeError::WrongType {
                want: "integer",
                got: self.kind(),
            }),
        }
    }

    pub const fn as_bytes(&self) -> Result<&[u8], BencodeError> {
        match self {
            Bencode::String(bytes) => Ok(*bytes),
            _ => Err(BencodeError::WrongType {
                want: "string",
                got: self.kind(),
            }),
        }
    }

    pub const fn as_dictionary(&self) -> Result<&BTreeMap<&[u8], Self>, BencodeError> {
        match self {
            Bencode::Dictionary(dict) => Ok(dict),
            _ => Err(BencodeError::WrongType {
                want: "dictionary",
                got: self.kind(),
            }),
        }
    }

    pub fn as_list(&self) -> Result<&[Self], BencodeError> {
        match self {
            Bencode::List(items) => Ok(items),
            _ => Err(BencodeError::WrongType {
                want: "list",
                got: self.kind(),
            }),
        }
    }

    fn write_at(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        match self {
            Bencode::Integer(n) => write!(f, "{n}"),

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

            Bencode::Dictionary(map) => {
                if map.is_empty() {
                    return write!(f, "{{}}");
                }

                writeln!(f, "{{")?;
                for (key, value) in map {
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

fn write_indent(f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        f.write_str(INDENT)?;
    }
    Ok(())
}

fn write_bytes(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    match std::str::from_utf8(bytes) {
        Ok(text) if text.len() <= MAX_STRING_LEN => {
            write!(f, "\"{text}\"")
        }
        Ok(text) => {
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

impl fmt::Display for Bencode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_at(f, 0)
    }
}
