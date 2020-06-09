//! Error types and error handling utilities.

use failure::Fail;
use prost::{DecodeError, EncodeError};
use std::convert::Infallible;

/// The error type for this crate.
///
/// It implements [failure::Fail] that it can be directly converted to
/// [failure::Error].
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(
        display = "checksum mismatch error: expect {}, but found {}",
        expect, found
    )]
    ChecksumMismatchError { expect: String, found: String },
    #[fail(display = "unexpected eof")]
    UnexpectedEofError,
    #[fail(display = "unicode error: {}", desc)]
    UnicodeError { desc: String },
    #[fail(display = "failed to decode example: {:?}", error)]
    ExampleDecodeError { error: DecodeError },
    #[fail(display = "failed to encode example: {:?}", error)]
    ExampleEncodeError { error: EncodeError },
    #[fail(display = "I/O error: {:?}", error)]
    IoError { error: std::io::Error },
    #[fail(display = "conversion error: {}", desc)]
    ConversionError { desc: String },
    #[fail(display = "invalid arguments: {}", desc)]
    InvalidArgumentsError { desc: String },
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
