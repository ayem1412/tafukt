use thiserror::Error;

use crate::protocol::decoder::error::DecoderError;

#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("could not calculate info_hash: {0}")]
    InfoHash(String),

    #[error("could not find an announce url")]
    NoAnnounce,

    #[error("could not parse url: {0}")]
    UrlParse(String),

    #[error("an error has occurred while handling the request")]
    RequestError(#[from] reqwest::Error),

    #[error("an error has occurred while trying to decode the response")]
    Bencode(#[from] DecoderError),

    #[error("expected {0}")]
    WrongBencodeType(String),

    #[error("response is invalid")]
    InvalidResponse,
}
