//! Tracking which pieces a peer has.
//!
//! Peers announce their pieces as a packed bit array — one bit per piece,
//! eight per byte — because a torrent can have tens of thousands of pieces
//! and a byte each would waste most of the message.

#[cfg(test)]
mod tests;

/// A peer sent a bitfield that does not match this torrent.
#[derive(Debug, thiserror::Error)]
pub enum BitfieldError {
    /// The byte count does not match the piece count.
    #[error("expected {expected} bytes for {piece_count} pieces, got {got}")]
    WrongLength {
        expected: usize,
        got: usize,
        piece_count: usize,
    },

    /// A bit past the last real piece was set.
    #[error("spare bits in the final byte must be zero")]
    SpareBitsSet,
}

/// Which pieces are present, one bit each.
///
/// Bits are numbered from the left of each byte, as the `BitTorrent` wire
/// format requires: piece 0 is the highest bit of the first byte, piece 7 the
/// lowest, piece 8 the highest of the second byte. This is the reverse of how
/// integers are normally laid out, and getting it backwards produces a
/// bitfield that looks valid but reports every piece in the wrong place.
///
/// # Example
///
/// ```
/// use bittorrent::bitfield::Bitfield;
///
/// let mut have = Bitfield::new(20);
/// have.set(0);
/// have.set(19);
///
/// assert!(have.has(0));
/// assert!(!have.has(5));
/// assert_eq!(have.count(), 2);
/// ```
pub struct Bitfield {
    /// The packed bits, eight pieces per byte.
    ///
    /// Length is [`piece_count`](Self::piece_count) rounded up to whole
    /// bytes, so the final byte may hold bits that correspond to no piece.
    ///
    /// Those spare bits must always be zero. The wire format requires it, and
    /// [`count`](Self::count) assumes it — a set spare bit would be counted
    /// as a piece that does not exist. Modifying this field directly can
    /// break that guarantee; prefer [`set`](Self::set), which cannot.
    bits: Vec<u8>,

    /// How many pieces the torrent actually has.
    ///
    /// Kept separately because the byte length only tells you the count
    /// rounded up to a multiple of eight — 2,801 pieces and 2,808 pieces
    /// both occupy 351 bytes.
    pub piece_count: usize,
}

impl Bitfield {
    /// An empty bitfield with room for `piece_count` pieces.
    #[must_use]
    pub fn new(piece_count: usize) -> Self {
        Self {
            // One bit per piece, rounded up to whole bytes.
            bits: vec![0; piece_count.div_ceil(8)],
            piece_count,
        }
    }

    /// Build a bitfield from the bytes a peer sent.
    ///
    /// # Errors
    ///
    /// Returns an error if the byte count is wrong for `piece_count`, or if
    /// any bit past the last real piece is set. Both mean the peer is
    /// confused or lying, so the connection should be dropped.
    pub fn from_bytes(bytes: &[u8], piece_count: usize) -> Result<Self, BitfieldError> {
        let expected = piece_count.div_ceil(8);

        if bytes.len() != expected {
            return Err(BitfieldError::WrongLength {
                expected,
                got: bytes.len(),
                piece_count,
            });
        }

        let used = piece_count.rem_euclid(8);

        if used != 0 {
            // Ones in every spare position: `used = 3` gives 0b0001_1111.
            let spare_mask = 0xFFu8 >> used;

            let has_spare_bits = bytes.last().is_some_and(|last| last & spare_mask != 0);

            if has_spare_bits {
                return Err(BitfieldError::SpareBitsSet);
            }
        }

        Ok(Self {
            bits: bytes.to_vec(),
            piece_count,
        })
    }

    /// The packed bits, for sending to a peer.
    #[must_use]
    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    /// Where a piece lives: which byte, and a mask for its bit within it.
    ///
    /// Returns `None` if the piece is out of range.
    const fn locate(&self, piece: usize) -> Option<(usize, u8)> {
        if piece >= self.piece_count {
            return None;
        }

        let byte_idx = piece.div_euclid(8);
        let mask = 1u8 << (7 - piece.rem_euclid(8));

        Some((byte_idx, mask))
    }

    /// Is this piece present?
    #[must_use]
    pub fn has(&self, piece: usize) -> bool {
        let Some((byte_idx, mask)) = self.locate(piece) else {
            return false;
        };

        self.bits.get(byte_idx).is_some_and(|byte| byte & mask != 0)
    }

    /// Mark a piece as present. Out-of-range pieces are ignored.
    pub fn set(&mut self, piece: usize) {
        let Some((byte_idx, mask)) = self.locate(piece) else {
            return;
        };

        if let Some(byte) = self.bits.get_mut(byte_idx) {
            *byte |= mask;
        }
    }

    /// How many pieces are marked present.
    #[must_use]
    pub fn count(&self) -> usize {
        self.bits
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }

    /// Whether every piece is present.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.count() == self.piece_count
    }
}
