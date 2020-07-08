//! Error types and error handling utilities.

use prost::{DecodeError, EncodeError};
use std::convert::Infallible;

/// The error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("checksum mismatch error: expect {expect:}, but found {found:}")]
    ChecksumMismatchError { expect: String, found: String },
    #[error("unexpected eof")]
    UnexpectedEofError,
    #[error("unicode error: {desc:}")]
    UnicodeError { desc: String },
    #[error("errored to decode example: {error:?}")]
    ExampleDecodeError { error: DecodeError },
    #[error("errored to encode example: {error:?}")]
    ExampleEncodeError { error: EncodeError },
    #[error("I/O error: {error:?}")]
    IoError { error: std::io::Error },
    #[error("conversion error: {desc:}")]
    ConversionError { desc: String },
    #[error("invalid arguments: {desc:}")]
    InvalidArgumentsError { desc: String },
    #[error("tch error: {desc:}")]
    TchError { desc: String },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError { error }
    }
}

impl From<EncodeError> for Error {
    fn from(error: EncodeError) -> Self {
        Self::ExampleEncodeError { error }
    }
}

impl From<DecodeError> for Error {
    fn from(error: DecodeError) -> Self {
        Self::ExampleDecodeError { error }
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!();
    }
}

#[cfg(feature = "with-tch")]
impl From<tch::TchError> for Error {
    fn from(error: tch::TchError) -> Self {
        Self::TchError {
            desc: format!("{:?}", error),
        }
    }
}
