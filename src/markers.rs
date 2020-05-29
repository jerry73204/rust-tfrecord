use crate::{error::Error, protos::Example};
use prost::Message;

pub trait GenericRecord
where
    Self: Sized,
{
    fn transform(bytes: Vec<u8>) -> Result<Self, Error>;
}

impl GenericRecord for Vec<u8> {
    fn transform(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(bytes)
    }
}

impl GenericRecord for Example {
    fn transform(bytes: Vec<u8>) -> Result<Self, Error> {
        let example = Example::decode(bytes.as_ref())?;
        Ok(example)
    }
}
