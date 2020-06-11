//! Marker traits.

use crate::{
    error::Error,
    protos::{Event, Example as RawExample},
    types::Example,
};
use prost::Message;

/// The trait marks the type that can be serailized to or deserialized from TFRecord raw bytes.
pub trait GenericRecord
where
    Self: Sized,
{
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error>;
    fn to_bytes(record: Self) -> Result<Vec<u8>, Error>;
}

impl GenericRecord for Vec<u8> {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(bytes)
    }

    fn to_bytes(record: Self) -> Result<Vec<u8>, Error> {
        Ok(record)
    }
}

impl GenericRecord for RawExample {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let example = RawExample::decode(bytes.as_ref())?;
        Ok(example)
    }

    fn to_bytes(record: Self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        RawExample::encode(&record, &mut bytes)?;
        Ok(bytes)
    }
}

impl GenericRecord for Example {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let raw_example = RawExample::decode(bytes.as_ref())?;
        let example = Example::from(raw_example);
        Ok(example)
    }

    fn to_bytes(example: Self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        let raw_example = RawExample::from(example);
        RawExample::encode(&raw_example, &mut bytes)?;
        Ok(bytes)
    }
}

impl GenericRecord for Event {
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
