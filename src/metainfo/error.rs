use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MetainfoError {
    #[error("expected a dictionary")]
    NotADictionary,

    #[error("not a valid UTF-8 string")]
    InvalidUtf8String,

    #[error("couldn't find the `{0}` key")]
    MissingKey(String),

    #[error("wrong value type for key: {0}")]
    WrongValueType(String),

    #[error("couldn't parse integer")]
    IntegerOverflow,

    #[error("pieces length must be multiple of 20")]
    InvalidPiecesLength,
}
