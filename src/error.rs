use failure::Fail;
use prost::DecodeError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(
        display = "checksum mismatch error: expect {}, but foudn {}",
        expect, found
    )]
    ChecksumMismatchError { expect: String, found: String },
    #[fail(display = "unexpected eof")]
    UnexpectedEofError,
    #[fail(display = "invalid options: {}", desc)]
    InvalidOptionsError { desc: String },
    #[fail(display = "failed to decode example: {:?}", error)]
    ExampleDecodeError { error: DecodeError },
    #[fail(display = "I/O error: {:?}", error)]
    IoError { error: std::io::Error },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError { error }
    }
}

impl From<DecodeError> for Error {
    fn from(error: DecodeError) -> Self {
        Self::ExampleDecodeError { error }
    }
}
