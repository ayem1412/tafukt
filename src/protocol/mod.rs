use std::collections::BTreeMap;

pub mod decoder;
use std::fmt::{self, Write};

#[derive(PartialEq, Clone)]
pub enum Bencode {
    /// Integers are represented by an 'i' followed by the number in base 10 followed by an 'e'.
    /// For example i3e corresponds to 3 and i-3e corresponds to -3.
    /// Integers have no size limitation. i-0e is invalid.
    /// All encodings with a leading zero, such as i03e, are invalid, other than i0e, which of
    /// course corresponds to 0.
    Integer(i64),

    /// Lists are encoded as an 'l' followed by their elements (also bencoded) followed by an 'e'.
    /// For example l4:spam4:eggse corresponds to ['spam', 'eggs'].
    List(Vec<Self>),

    String(Vec<u8>),

    /// Dictionaries are encoded as a 'd' followed by a list of alternating keys and their
    /// corresponding values followed by an 'e'.
    /// For example, d3:cow3:moo4:spam4:eggse corresponds to {'cow': 'moo', 'spam': 'eggs'}
    /// and d4:spaml1:a1:bee corresponds to {'spam': ['a', 'b']}.
    /// Keys must be strings and appear in sorted order (sorted as raw strings, not alphanumerics).
    Dictionary(BTreeMap<String, Self>),
}

impl fmt::Debug for Bencode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bencode::Integer(value) => write!(f, "Integer({})", value),
            Bencode::String(bytes) => match std::str::from_utf8(bytes) {
                Ok(utf8) => write!(f, "String({:?})", utf8),
                Err(_) => write!(f, "String({:?})", bytes),
            },
            Bencode::List(items) => f.debug_tuple("List").field(items).finish(),
            Bencode::Dictionary(map) => f.debug_tuple("Dictionary").field(map).finish(),
        }
    }
}

impl fmt::Display for Bencode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_pretty(f, 0)
    }
}

impl Bencode {
    fn write_pretty(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let pad = "  ".repeat(indent);
        let child_pad = "  ".repeat(indent + 2);

        match self {
            Bencode::Integer(value) => write!(f, "{}", value),
            Bencode::String(bytes) => {
                if let Ok(utf8) = std::str::from_utf8(bytes) {
                    escape_json_string(f, utf8)
                } else {
                    write!(f, "[")?;
                    for (i, &byte) in bytes.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", byte)?;
                    }
                    write!(f, "]")
                }
            },
            Bencode::List(items) => {
                if items.is_empty() {
                    return write!(f, "[]");
                }
                writeln!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{}", child_pad)?;
                    item.write_pretty(f, indent + 2)?;
                    if i + 1 < items.len() {
                        write!(f, ",")?;
                    }
                    writeln!(f)?;
                }
                write!(f, "{}", pad)?;
                write!(f, "]")
            },
            Bencode::Dictionary(map) => {
                if map.is_empty() {
                    return write!(f, "{{}}");
                }
                writeln!(f, "{{")?;
                let len = map.len();
                for (i, (key, value)) in map.iter().enumerate() {
                    write!(f, "{}", child_pad)?;
                    escape_json_string(f, key)?;
                    write!(f, ": ")?;
                    value.write_pretty(f, indent + 2)?;
                    if i + 1 < len {
                        write!(f, ",")?;
                    }
                    writeln!(f)?;
                }
                write!(f, "{}", pad)?;
                write!(f, "}}")
            },
        }
    }
}

fn escape_json_string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    write!(f, "\"")?;
    for c in s.chars() {
        match c {
            '"' => write!(f, "\\\"")?,
            '\\' => write!(f, "\\\\")?,
            '\n' => write!(f, "\\n")?,
            '\r' => write!(f, "\\r")?,
            '\t' => write!(f, "\\t")?,
            c if c.is_control() => write!(f, "\\u{:04x}", c as u32)?,
            c => f.write_char(c)?,
        }
    }
    write!(f, "\"")
}
