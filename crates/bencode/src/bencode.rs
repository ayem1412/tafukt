use std::{collections::BTreeMap, fmt};

const MAX_STRING_LEN: usize = 60;
const INDENT: &str = "  ";

#[derive(Debug)]
pub enum Bencode<'a> {
    String(&'a [u8]),
    Integer(i64),
    Dictionary(BTreeMap<&'a [u8], Bencode<'a>>),
    List(Vec<Bencode<'a>>),
}

impl fmt::Display for Bencode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_at(f, 0)
    }
}

impl Bencode<'_> {
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
                .map(|(i, _)| i)
                .unwrap_or(0);
            write!(f, "\"{}…\" ({} bytes)", &text[..cut], bytes.len())
        }
        Err(_) => write!(f, "<{} bytes binary>", bytes.len()),
    }
}
