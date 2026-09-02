//! A bounds-checked cursor for walking a byte buffer.
//!
//! Every read is checked, so a truncated or hostile input produces an error
//! rather than a panic. Nesting depth is tracked here too, so deeply nested
//! bencode cannot overflow the stack during recursive parsing.

use thiserror::Error;

/// How deeply values may nest before parsing is refused.
///
/// Real torrents nest about four levels. The limit exists to stop a hostile
/// input of nothing but `l` bytes from overflowing the stack — a crash that
/// cannot be caught.
pub const DEPTH_MAX: u16 = 100;

/// A position within a byte buffer, plus the current nesting depth.
///
/// The lifetime ties returned slices to the original buffer, so [`take`]
/// borrows rather than copies.
///
/// [`take`]: Cursor::take
pub struct Cursor<'a> {
    /// The buffer being read.
    data: &'a [u8],

    /// Index of the next unread byte.
    pos: usize,

    /// How many containers are currently open.
    depth: u16,
}

/// A read went past the end of the buffer, or the input was malformed.
#[derive(Debug, Error)]
pub enum CursorError {
    /// A read ran past the end of the buffer — usually a truncated file.
    #[error("unexpected end of input")]
    Eof,

    /// A length was large enough to overflow when added to the position.
    #[error("length overflow")]
    Overflow,

    /// A specific byte was required and a different one was found.
    #[error("expected byte {want:?}, got {got:?}")]
    Unexpected {
        /// The byte the format required.
        want: u8,
        /// The byte actually present.
        got: u8,
    },

    /// Values nested past [`DEPTH_MAX`].
    #[error("nesting too deep (limit {})", DEPTH_MAX)]
    TooDeep,
}

impl<'a> Cursor<'a> {
    /// Start reading `data` from the beginning.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            depth: 0,
        }
    }

    /// Index of the next unread byte.
    ///
    /// Read this before and after parsing a value to record the byte range
    /// that value occupied.
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Whether every byte has been consumed.
    ///
    /// Check this after parsing a top-level value to detect trailing data.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Return the next byte without consuming it.
    ///
    /// # Errors
    ///
    /// Returns [`CursorError::Eof`] at the end of the buffer.
    pub fn peek(&self) -> Result<u8, CursorError> {
        self.data.get(self.pos).copied().ok_or(CursorError::Eof)
    }

    /// Consume and return the next byte.
    ///
    /// # Errors
    ///
    /// Returns [`CursorError::Eof`] at the end of the buffer.
    pub fn bump(&mut self) -> Result<u8, CursorError> {
        let byte = self.peek()?;
        self.pos += 1;
        Ok(byte)
    }

    /// Consume exactly `n` bytes and return a borrow of them.
    ///
    /// Nothing is copied — the returned slice points into the original buffer.
    ///
    /// # Errors
    ///
    /// Returns [`CursorError::Overflow`] if `n` is large enough to overflow
    /// the position, and [`CursorError::Eof`] if fewer than `n` bytes remain.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], CursorError> {
        let end = self.pos.checked_add(n).ok_or(CursorError::Overflow)?;
        let slice = self.data.get(self.pos..end).ok_or(CursorError::Eof)?;
        self.pos = end;
        Ok(slice)
    }

    /// Consume the next byte, requiring it to be `want`.
    ///
    /// # Errors
    ///
    /// Returns [`CursorError::Unexpected`] if a different byte is there, or
    /// [`CursorError::Eof`] at the end of the buffer.
    pub fn expect(&mut self, want: u8) -> Result<(), CursorError> {
        let got = self.bump()?;

        if got == want {
            Ok(())
        } else {
            Err(CursorError::Unexpected { want, got })
        }
    }

    /// Record entering a nested container, refusing to go past [`DEPTH_MAX`].
    ///
    /// Call this before recursing, and pair it with [`leave`](Cursor::leave).
    ///
    /// # Errors
    ///
    /// Returns [`CursorError::TooDeep`] once the limit is reached.
    pub const fn enter(&mut self) -> Result<(), CursorError> {
        self.depth += 1;

        if self.depth > DEPTH_MAX {
            return Err(CursorError::TooDeep);
        }

        Ok(())
    }

    /// Record leaving a nested container.
    ///
    /// Saturates at zero, so an unbalanced call cannot panic.
    pub const fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}
