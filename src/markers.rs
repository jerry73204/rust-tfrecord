//! Marker traits.

use crate::{
    error::Error,
    protobuf::{summary::Image, Event, Example as RawExample},
    types::Example,
};
use prost::Message;

/// Mark types the is serailized to or deserialized from TFRecord format.
pub trait GenericRecord
where
    Self: Sized,
{
    /// Deserialze from bytes in TFRecord format.
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error>;
    /// Serialze to bytes in TFRecord format.
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

/// A trait marking if the type can be converted to a list of imgaes.
pub trait TryInfoImageList {
    type Error;

    fn try_into_image_list(self) -> Result<Vec<Image>, Self::Error>;
}
