use std::collections::BTreeMap;
use std::fmt;

const MAX_STRING_LEN: usize = 60;
const INDENT: &str = "  ";

pub enum BencodeValue<'a> {
    String(&'a [u8]),
    Integer(i64),
    Dictionary(BTreeMap<&'a [u8], BencodeValue<'a>>),
    List(Vec<BencodeValue<'a>>),
}

impl fmt::Display for BencodeValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_at(f, 0)
    }
}

impl BencodeValue<'_> {
    fn write_at(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        match self {
            BencodeValue::Integer(n) => write!(f, "{n}"),

            BencodeValue::String(bytes) => write_bytes(f, bytes),

            BencodeValue::List(items) => {
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

            BencodeValue::Dictionary(map) => {
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
