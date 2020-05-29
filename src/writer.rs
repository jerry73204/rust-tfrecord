use crate::{error::Error, markers::GenericRecord, protos::Example};
use futures::io::AsyncWrite;
use std::{io::Write, marker::PhantomData, path::Path};

pub type BytesWriter<W> = RecordWriter<Vec<u8>, W>;
pub type ExampleWriter<W> = RecordWriter<Example, W>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordWriterInit;

impl RecordWriterInit {
    pub fn from_writer<T, W>(writer: W) -> Result<RecordWriter<T, W>, Error>
    where
        T: GenericRecord,
        W: Write,
    {
        Ok(RecordWriter {
            writer,
            _phantom: PhantomData,
        })
    }

    pub fn create<T, P>(
        path: P,
    ) -> Result<RecordWriter<T, std::io::BufWriter<std::fs::File>>, Error>
    where
        T: GenericRecord,
        P: AsRef<Path>,
    {
        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        Self::from_writer(writer)
    }

    pub fn from_async_writer<T, W>(writer: W) -> Result<RecordWriter<T, W>, Error>
    where
        T: GenericRecord,
        W: AsyncWrite,
    {
        Ok(RecordWriter {
            writer,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordWriter<T, W>
where
    T: GenericRecord,
{
    writer: W,
    _phantom: PhantomData<T>,
}

impl<T, W> RecordWriter<T, W>
where
    T: GenericRecord,
    W: Write,
{
    pub fn send(&mut self, record: T) -> Result<(), Error> {
        let bytes = T::to_bytes(record)?;
        crate::io::blocking::try_write_record(&mut self.writer, bytes)?;
        Ok(())
    }
}

impl<T, W> RecordWriter<T, W>
where
    T: GenericRecord,
    W: AsyncWrite + Unpin,
{
    pub async fn send_async(&mut self, record: T) -> Result<(), Error> {
        let bytes = T::to_bytes(record)?;
        crate::io::async_::try_write_record(&mut self.writer, bytes).await?;
        Ok(())
    }
}
