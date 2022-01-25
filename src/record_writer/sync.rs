use crate::{error::Result, markers::GenericRecord, protobuf::Example};
use std::{
    fs::File,
    io::{BufWriter, Write},
    marker::PhantomData,
    path::Path,
};

/// Alias to [RecordWriter] which input record type [Vec<u8>](Vec).
pub type BytesWriter<W> = RecordWriter<Vec<u8>, W>;

/// Alias to [RecordWriter] which input record type [Example].
pub type ExampleWriter<W> = RecordWriter<Example, W>;

/// The record writer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordWriter<T, W>
where
    T: GenericRecord,
{
    writer: W,
    _phantom: PhantomData<T>,
}

impl<T> RecordWriter<T, BufWriter<File>>
where
    T: GenericRecord,
{
    /// Build a writer writing to a new file.
    pub fn create<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path)?);
        Self::from_writer(writer)
    }
}

impl<T, W> RecordWriter<T, W>
where
    T: GenericRecord,
    W: Write,
{
    /// Build a writer from a writer with [Write] trait.
    pub fn from_writer(writer: W) -> Result<Self> {
        Ok(Self {
            writer,
            _phantom: PhantomData,
        })
    }

    /// Write a record.
    ///
    /// The method is enabled if the underlying writer implements [Write].
    pub fn send(&mut self, record: T) -> Result<()> {
        let bytes = T::to_bytes(record)?;
        crate::io::sync::try_write_record(&mut self.writer, bytes)?;
        Ok(())
    }

    /// Flush the output stream.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
