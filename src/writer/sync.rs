use super::RecordWriter;
use crate::{error::Result, markers::GenericRecord};
use std::{
    fs::File,
    io::{BufWriter, Write},
    marker::PhantomData,
    path::Path,
};

impl<T> RecordWriter<T, BufWriter<File>>
where
    T: GenericRecord,
{
    /// Construct a [RecordWriter] by creating a file at specified path.
    ///
    /// The constructed [RecordWriter] enables the blocking [send](RecordWriter::send) method.
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
    /// Construct a [RecordWriter] from a type with [Write] trait.
    ///
    /// The constructed [RecordWriter] enables the blocking [send](RecordWriter::send) method.
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
