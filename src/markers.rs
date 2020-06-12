//! Marker traits.

use crate::{
    error::Error,
    protos::{DataType, Event, Example as RawExample},
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

/// The marker trait that can be converted to elements of [TensorProto](crate::protos::TensorProto).
pub trait TensorProtoElement
where
    Self: Copy,
{
    const DATA_TYPE: DataType;

    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_to_le_bytes {
    ($ty:ty, $dtype:expr) => {
        impl TensorProtoElement for $ty {
            const DATA_TYPE: DataType = $dtype;

            fn to_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().iter().cloned().collect()
            }
        }
    };
}

impl_to_le_bytes!(u8, DataType::DtUint8);
impl_to_le_bytes!(u16, DataType::DtUint16);
impl_to_le_bytes!(u32, DataType::DtUint32);
impl_to_le_bytes!(u64, DataType::DtUint64);
impl_to_le_bytes!(i8, DataType::DtInt8);
impl_to_le_bytes!(i16, DataType::DtInt16);
impl_to_le_bytes!(i32, DataType::DtInt32);
impl_to_le_bytes!(i64, DataType::DtInt64);
impl_to_le_bytes!(f32, DataType::DtFloat);
impl_to_le_bytes!(f64, DataType::DtDouble);

/// A trait marking types that can be converted to elements of [HistogramProto](crate::protos::HistogramProto).
pub trait HistogramProtoElement
where
    Self: Copy,
{
    fn to_f64(&self) -> f64;
}

impl HistogramProtoElement for u8 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for u16 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for u32 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for u64 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for i8 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for i16 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for i32 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for i64 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for f32 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl HistogramProtoElement for f64 {
    fn to_f64(&self) -> f64 {
        *self
    }
}
