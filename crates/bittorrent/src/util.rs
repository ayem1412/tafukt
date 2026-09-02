//! Shared helpers for reading torrent metadata.
//!
//! Mostly small wrappers around bencode dictionary lookups, plus the path
//! validation that keeps a hostile torrent from writing outside its download
//! directory.

use std::path::PathBuf;

use bencode::bencode::Bencode;

use crate::{
    hex,
    metainfo::{Dict, MetainfoError},
};

/// Look up a key the format requires.
///
/// # Errors
///
/// Returns [`MetainfoError::KeyNotFound`] if the key is absent.
pub fn get_key<'a>(
    dict: &'a Dict<'a>,
    key: &'static str,
) -> Result<&'a Bencode<'a>, MetainfoError> {
    dict.get(key.as_bytes())
        .ok_or(MetainfoError::KeyNotFound(key))
}

/// Look up a key the format allows to be absent.
#[must_use]
pub fn get_opt<'a>(dict: &'a Dict<'a>, key: &'static str) -> Option<&'a Bencode<'a>> {
    dict.get(key.as_bytes())
}

/// Look up an optional key and read it as text.
///
/// The conversion is lossy, which is safe for URLs but would mangle a
/// filename in an unknown encoding — keep those as bytes instead.
///
/// # Errors
///
/// Returns an error if the key is present but is not a byte string. A missing
/// key gives `Ok(None)`.
pub fn get_opt_string_lossy<'a>(
    dict: &'a Dict<'a>,
    key: &'static str,
) -> Result<Option<String>, MetainfoError> {
    match dict.get(key.as_bytes()) {
        Some(value) => Ok(Some(
            String::from_utf8_lossy(value.as_bytes()?).into_owned(),
        )),
        None => Ok(None),
    }
}

/// Reject a path component that could escape the download directory.
///
/// A legitimate torrent never contains any of these, so a match means the
/// file is either corrupt or hostile. The whole torrent is rejected rather
/// than sanitised: rewriting a path silently could collapse two files onto
/// one another, or leave the file count disagreeing with the torrent's own.
///
/// Separators are checked with `contains` rather than equality, since a
/// component like `foo/../bar` is dangerous without equalling anything.
///
/// # Errors
///
/// Returns [`MetainfoError::InvalidPath`] if the component is empty, is `.`
/// or `..`, or contains a separator or null byte.
pub fn check_path_component(component: &[u8]) -> Result<(), MetainfoError> {
    if component.is_empty()
        || component == b"."
        || component == b".."
        || component.contains(&b'/')
        || component.contains(&b'\\')
        // A null can truncate a path once it reaches the filesystem, so a
        // name like `safe.txt\0/../evil` could behave differently there.
        || component.contains(&0)
    {
        return Err(MetainfoError::InvalidPath);
    }

    Ok(())
}

/// Read a bencode integer as an unsigned size.
///
/// Bencode integers are signed, so this is where a negative length is caught.
///
/// # Errors
///
/// Returns an error if the value is not an integer, or is negative.
pub fn as_u64(value: &Bencode) -> Result<u64, MetainfoError> {
    u64::try_from(value.as_i64()?).map_err(|_| MetainfoError::NegativeLength)
}

/// Convert raw name bytes into a path.
///
/// Lossy, because torrent names have no guaranteed encoding and a filesystem
/// path has to be *some* concrete string. Validate components with
/// [`check_path_component`] before calling this.
#[must_use]
pub fn to_path(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}

/// Decode percent-encoding, including the `+` means space convention.
///
/// That second rule comes from HTML forms and is inherited by magnet links —
/// without it, every display name reads with pluses where spaces belong.
///
/// Decoding is lenient: a malformed escape such as `%ZZ`, or a `%` too close
/// to the end, is kept literally rather than dropped. Nothing is ever
/// silently lost.
///
/// Returns bytes rather than text, since the decoded content may be in any
/// encoding.
#[must_use]
pub fn percent_decode(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'+' => {
                decoded.push(b' ');
                i += 1;
            }

            b'%' => {
                // `get` returns None past the end, so an incomplete escape
                // falls through to the same branch as an invalid one.
                if let (Some(Ok(high)), Some(Ok(low))) = (
                    bytes.get(i + 1).copied().map(hex::hex_value),
                    bytes.get(i + 2).copied().map(hex::hex_value),
                ) {
                    decoded.push(high << 4 | low);
                    i += 3;
                } else {
                    decoded.push(b'%');
                    i += 1;
                }
            }

            other => {
                decoded.push(other);
                i += 1;
            }
        }
    }

    decoded
}
