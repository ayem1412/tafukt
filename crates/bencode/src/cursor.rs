use thiserror::Error;

/// Cursor used by the Parser to traverse bytes.
pub struct Cursor<'a> {
    /// Current position's data.
    data: &'a [u8],
    /// Current position.
    pos: usize,
}

#[derive(Debug, Error)]
pub enum CursorError {
    #[error("end of file")]
    Eof,
    #[error("overflow")]
    Overflow,
    #[error("expected {want} got {got}")]
    Unexpected { want: u8, got: u8 },
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Look at the current byte without consuming it.
    pub fn peek(&self) -> Result<u8, CursorError> {
        self.data.get(self.pos).copied().ok_or(CursorError::Eof)
    }

    /// Consume the current byte, bump the position by 1.
    pub fn bump(&mut self) -> Result<u8, CursorError> {
        let nb = self.peek()?;
        self.pos += 1;
        Ok(nb)
    }

    /// Consume exactly `n` bytes and return a slice of them.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], CursorError> {
        let end = self.pos.checked_add(n).ok_or(CursorError::Overflow)?;
        let slice = self.data.get(self.pos..end).ok_or(CursorError::Eof)?;
        self.pos = end;

        Ok(slice)
    }

    /// Expect a specific byte next, error if it's anything else.
    pub fn expect(&mut self, want: u8) -> Result<(), CursorError> {
        let got = self.bump()?;
        if got == want {
            Ok(())
        } else {
            Err(CursorError::Unexpected { want, got })
        }
    }
}
