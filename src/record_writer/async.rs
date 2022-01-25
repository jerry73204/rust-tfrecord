use crate::{
    error::{Error, Result},
    markers::GenericRecord,
    protobuf::Example,
};
use async_std::{fs::File, io::BufWriter, path::Path};
use futures::{
    io::{AsyncWrite, AsyncWriteExt as _},
    sink,
    sink::Sink,
};
use std::marker::PhantomData;

/// Alias to [RecordAsyncWriter] which input record type [Vec<u8>](Vec).
pub type BytesAsyncWriter<W> = RecordAsyncWriter<Vec<u8>, W>;

/// Alias to [RecordAsyncWriter] which input record type [Example].
pub type ExampleAsyncWriter<W> = RecordAsyncWriter<Example, W>;

/// The record writer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordAsyncWriter<T, W>
where
    T: GenericRecord,
{
    writer: W,
    _phantom: PhantomData<T>,
}

impl<T> RecordAsyncWriter<T, BufWriter<File>>
where
    T: GenericRecord,
{
    /// Build a writer writing to a new file.
    pub async fn create<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path).await?);
        Self::from_writer(writer)
    }
}

impl<T, W> RecordAsyncWriter<T, W>
where
    T: GenericRecord,
    W: AsyncWrite + Unpin,
{
    /// Build a writer from a writer with [AsyncWrite] trait.
    pub fn from_writer(writer: W) -> Result<Self> {
        Ok(Self {
            writer,
            _phantom: PhantomData,
        })
    }

    /// Write a record.
    pub async fn send(&mut self, record: T) -> Result<()> {
        let bytes = T::to_bytes(record)?;
        crate::io::r#async::try_write_record(&mut self.writer, bytes).await?;
        Ok(())
    }

    /// Flush the output stream asynchronously.
    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await?;
        Ok(())
    }

    /// Convert into a [Sink].
    pub fn into_sink(self) -> impl Sink<T, Error = Error> {
        sink::unfold(self, |mut writer, record| async move {
            writer.send(record).await?;
            Ok(writer)
        })
    }
}
