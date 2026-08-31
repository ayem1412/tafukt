#[derive(Debug, thiserror::Error)]
pub enum HexError {
    #[error("hex string must have an even number of characters")]
    OddLength,
    #[error("invalid hex character: {0:?}")]
    InvalidCharacter(char),
}

pub const fn hex_value(byte: u8) -> Result<u8, HexError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        other => Err(HexError::InvalidCharacter(other as char)),
    }
}

pub fn decode_hex_array<const N: usize>(s: &str) -> Result<[u8; N], HexError> {
    let bytes = s.as_bytes();

    if bytes.len() != N * 2 {
        return Err(HexError::OddLength);
    }

    let mut out = [0u8; N];
    for (slot, pair) in out.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *slot = hex_value(pair[0])? << 4 | hex_value(pair[1])?;
    }

    Ok(out)
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}
