use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("no bytes to read")]
    Empty,

    #[error("unexpected extra data")]
    UnexpectedExtraData,

    #[error("invalid integer syntax")]
    InvalidIntegerSyntax,

    #[error("invalid byte for type: {0}")]
    InvalidByte(String),

    #[error("integer is leading with zeros")]
    IntegerLeadingZero,

    #[error("integer is negative zero")]
    IntegerNegativeZero,
}
