use super::RecordReaderConfig;
use crate::{
    error::Result,
    markers::GenericRecord,
    protobuf::{Event, Example as RawExample},
    types::Example,
};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    marker::PhantomData,
    path::Path,
};

pub type BytesIter<R> = RecordIter<Vec<u8>, R>;

pub type RawExampleIter<R> = RecordIter<RawExample, R>;

pub type ExampleIter<R> = RecordIter<Example, R>;

pub type EventIter<R> = RecordIter<Event, R>;

pub struct RecordIter<T, R>
where
    T: GenericRecord,
    R: Read,
{
    reader: Option<R>,
    check_integrity: bool,
    _phantom: PhantomData<T>,
}

impl<T, R> RecordIter<T, R>
where
    T: GenericRecord,
    R: Read,
{
    /// Construct a [RecordReader] from a reader type implementing [Read](std::io::Read).
    pub fn from_reader(reader: R, config: RecordReaderConfig) -> Self {
        let RecordReaderConfig { check_integrity } = config;

        Self {
            reader: Some(reader),
            check_integrity,
            _phantom: PhantomData,
        }
    }
}

impl<T> RecordIter<T, BufReader<File>>
where
    T: GenericRecord,
{
    /// Construct a [RecordReader] from a path.
    pub fn open<P>(path: P, config: RecordReaderConfig) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let record_reader = Self::from_reader(reader, config);
        Ok(record_reader)
    }
}

impl<T, R> Iterator for RecordIter<T, R>
where
    T: GenericRecord,
    R: Read,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader = self.reader.as_mut()?;
        let bytes: Option<Result<_>> =
            crate::io::sync::try_read_record(reader, self.check_integrity).transpose();

        if bytes.is_none() {
            self.reader = None;
        }
        let record = bytes?.and_then(T::from_bytes);
        Some(record)
    }
}
