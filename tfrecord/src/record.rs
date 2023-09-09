//! Marker traits.

use crate::{
    error::Error,
    protobuf::{Event, Example},
};
use prost::Message as _;

/// Mark types the is serailized to or deserialized from TFRecord format.
pub trait Record
where
    Self: Sized,
{
    /// Deserialze from bytes in TFRecord format.
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error>;
    /// Serialze to bytes in TFRecord format.
    fn to_bytes(record: Self) -> Result<Vec<u8>, Error>;
}

impl Record for Vec<u8> {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(bytes)
    }

    fn to_bytes(record: Self) -> Result<Vec<u8>, Error> {
        Ok(record)
    }
}

impl Record for Example {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let example = Example::decode(bytes.as_ref())?;
        Ok(example)
    }

    fn to_bytes(record: Self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        Example::encode(&record, &mut bytes)?;
        Ok(bytes)
    }
}

impl Record for Event {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let example = Event::decode(bytes.as_ref())?;
        Ok(example)
    }

    fn to_bytes(record: Self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        Event::encode(&record, &mut bytes)?;
        Ok(bytes)
    }
}
