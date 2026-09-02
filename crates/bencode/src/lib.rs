//! A bencode encoder and decoder.
//!
//! Bencode is the data format behind `.torrent` files and the `BitTorrent`
//! DHT. It has four types: byte strings, integers, lists, and dictionaries.
//!
//! # Quick start
//!
//! ```
//! use bencode::{decoder::decode, encoder::encode};
//!
//! let value = decode(b"d3:cow3:moo4:spam4:eggse")?;
//!
//! let entries = value.as_dictionary()?;
//! assert_eq!(entries[b"cow".as_slice()].as_bytes()?, b"moo");
//!
//! // Decoding and re-encoding reproduces the input exactly.
//! assert_eq!(encode(&value), b"d3:cow3:moo4:spam4:eggse");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Borrowing
//!
//! Decoded values borrow from the input buffer rather than copying it, so
//! the buffer must outlive them. That keeps decoding cheap even for torrents
//! with tens of thousands of piece hashes.
//!
//! # Byte strings are not text
//!
//! [`Bencode::String`] holds arbitrary bytes.
//! Torrent files store filenames in unknown encodings and raw SHA-1 hashes
//! as strings, so neither is guaranteed to be valid UTF-8. Convert to text
//! only where you know the encoding, and prefer a lossy conversion for
//! anything a user will see.
//!
//! # Strictness and round-tripping
//!
//! Decoding rejects input that some decoders tolerate — leading zeros,
//! negative zero, empty numbers, trailing data. Combined with sorted
//! dictionary keys, this guarantees that decoding and re-encoding gives back
//! the original bytes.
//!
//! That property matters for `BitTorrent`: a torrent's identity is a hash of
//! its raw `info` dictionary, and a re-encoding that differs by one byte
//! produces a hash no peer will accept. For that case, see
//! [`decode_dictionary_with_spans`],
//! which reports where each value's original bytes live so you can hash them
//! directly.
//!
//! # Hostile input
//!
//! Every entry point is safe to point at bytes from a stranger. Malformed
//! input returns an error, reads are bounds-checked, and nesting is
//! depth-limited so a deeply nested value cannot overflow the stack.

/// The [`Bencode`](bencode::Bencode) value type and its accessors.
pub mod bencode;

/// A bounds-checked cursor over a byte buffer.
pub mod cursor;

/// Turning bytes into [`Bencode`](bencode::Bencode) values.
pub mod decoder;

/// Turning [`Bencode`](bencode::Bencode) values back into bytes.
pub mod encoder;

pub use bencode::{Bencode, BencodeError};
pub use decoder::{DecoderError, decode, decode_dictionary_with_spans};
pub use encoder::{encode, encode_into};
