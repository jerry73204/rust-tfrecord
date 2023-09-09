//! Error types and error handling utilities.

use std::{borrow::Cow, convert::Infallible};

/// The result with error type defaults to [Error].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("checksum mismatch error: expect {expect:#010x}, but found {found:#010x}")]
    ChecksumMismatch { expect: u32, found: u32 },
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("errored to decode example: {0}")]
    ExampleDecodeError(prost::DecodeError),
    #[error("errored to encode example: {0}")]
    ExampleEncodeError(prost::EncodeError),
    #[error("I/O error: {0}")]
    IoError(std::io::Error),
    #[error("conversion error: {desc:}")]
    ConversionError { desc: Cow<'static, str> },
    #[error("invalid arguments: {desc:}")]
    InvalidArgumentsError { desc: Cow<'static, str> },
    #[cfg(feature = "with-tch")]
    #[error("tch error: {0}")]
    TchError(tch::TchError),
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn conversion(desc: impl Into<Cow<'static, str>>) -> Self {
        Self::ConversionError { desc: desc.into() }
    }

    pub(crate) fn invalid_argument(desc: impl Into<Cow<'static, str>>) -> Self {
        Self::ConversionError { desc: desc.into() }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<prost::EncodeError> for Error {
    fn from(error: prost::EncodeError) -> Self {
        Self::ExampleEncodeError(error)
    }
}

impl From<prost::DecodeError> for Error {
    fn from(error: prost::DecodeError) -> Self {
        Self::ExampleDecodeError(error)
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
        Self::TchError(error)
    }
}

macro_rules! ensure_argument {
    ($cond:expr, $($arg:tt) *) => {
        if !$cond {
            return Err(crate::error::Error::invalid_argument(format!( $($arg)* )));
        }
    };
}
pub(crate) use ensure_argument;
