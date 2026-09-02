//! Hex encoding and decoding.
//!
//! `BitTorrent` writes infohashes as hex in magnet links and anywhere a hash is
//! shown to a user. Twenty raw bytes become forty characters.

/// Hex text could not be decoded.
#[derive(Debug, thiserror::Error)]
pub enum HexError {
    /// The input was not the length the caller required.
    #[error("expected {expected} hex characters, got {got}")]
    WrongLength {
        /// Characters needed for the requested byte count.
        expected: usize,
        /// Characters actually supplied.
        got: usize,
    },

    /// A byte outside `0-9`, `a-f`, and `A-F` appeared.
    #[error("invalid hex character: {0:?}")]
    InvalidCharacter(char),
}

/// Convert one hex character to its value, 0 through 15.
///
/// Both cases are accepted, since magnet links in the wild use either.
///
/// # Errors
///
/// Returns [`HexError::InvalidCharacter`] for anything that is not a hex digit.
pub const fn hex_value(byte: u8) -> Result<u8, HexError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        other => Err(HexError::InvalidCharacter(other as char)),
    }
}

/// Decode hex text into exactly `N` bytes.
///
/// The length is fixed by the type, so no `Vec` is allocated and no
/// conversion is needed afterwards. For a 20-byte infohash, call it as
/// `decode_hex_array::<20>(text)` or let the target type infer `N`.
///
/// # Errors
///
/// Returns [`HexError::WrongLength`] unless the input is exactly `N * 2`
/// characters, and [`HexError::InvalidCharacter`] for a non-hex byte.
///
/// # Example
///
/// ```
/// use bittorrent::util::hex::decode_hex_array;
///
/// let hash: [u8; 4] = decode_hex_array("deadbeef")?;
/// assert_eq!(hash, [0xde, 0xad, 0xbe, 0xef]);
/// # Ok::<(), bittorrent::util::hex::HexError>(())
/// ```
pub fn decode_hex_array<const N: usize>(s: &str) -> Result<[u8; N], HexError> {
    let bytes = s.as_bytes();
    let expected = N * 2;

    if bytes.len() != expected {
        return Err(HexError::WrongLength {
            expected,
            got: bytes.len(),
        });
    }

    let mut out = [0u8; N];

    // The length check above guarantees no remainder, so `.0` covers everything.
    for (slot, pair) in out.iter_mut().zip(bytes.as_chunks::<2>().0) {
        // First character is the high nibble, second the low.
        *slot = hex_value(pair[0])? << 4 | hex_value(pair[1])?;
    }

    Ok(out)
}

/// Encode bytes as lowercase hex text.
///
/// A 20-byte infohash becomes the 40-character form trackers and torrent
/// sites display.
///
/// # Example
///
/// ```
/// use bittorrent::util::hex::to_hex;
///
/// assert_eq!(to_hex(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
/// ```
#[must_use]
pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        // Writing to a String is infallible; the Result satisfies fmt::Write.
        let _ = write!(out, "{byte:02x}");
    }

    out
}
