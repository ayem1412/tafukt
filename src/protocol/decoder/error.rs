use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum DecoderError {
    #[error("no bytes to read")]
    Empty,

    #[error("unknown type {0}")]
    UnknownType(String),

    #[error("unexpected extra data")]
    UnexpectedExtraData,

    #[error("no terminator found")]
    MissingTerminator,

    #[error("invalid integer syntax")]
    InvalidIntegerSyntax,

    #[error("invalid byte for type: {0}")]
    InvalidByte(u8),

    #[error("integer is leading with zeros")]
    IntegerLeadingZero,

    #[error("integer is negative zero")]
    IntegerNegativeZero,

    #[error("string length mismatch: expected {0} bytes")]
    StringInvalidLength(usize),

    #[error("string is not valid UTF-8")]
    StringInvalidUtf8,

    #[error("expected dictionnary key to be string")]
    DictionaryInvalidKeyType,
}
