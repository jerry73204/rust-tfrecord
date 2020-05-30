use crate::{error::Error, protos::Example, record::EasyExample};
use prost::Message;

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

impl GenericRecord for Example {
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

impl GenericRecord for EasyExample {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        let example = Example::decode(bytes.as_ref())?;
        let easy_example = EasyExample::from(example);
        Ok(easy_example)
    }

    fn to_bytes(easy_example: Self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        let example = Example::from(easy_example);
        Example::encode(&example, &mut bytes)?;
        Ok(bytes)
    }
}
