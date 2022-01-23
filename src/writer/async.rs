#![cfg(feature = "async")]

use super::RecordWriter;
use crate::{
    error::{Error, Result},
    markers::GenericRecord,
};
use async_std::{fs::File, io::BufWriter, path::Path};
use futures::{io::AsyncWriteExt, sink, sink::Sink};
use std::marker::PhantomData;

impl<T> RecordWriter<T, BufWriter<File>>
where
    T: GenericRecord,
{
    /// Construct a [RecordWriter] by creating a file at specified path.
    ///
    /// The constructed [RecordWriter] enables the asynchronous [send_async](RecordWriter::send_async) method.
    pub async fn create_async<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path).await?);
        Self::from_async_writer(writer)
    }
}

impl<T, W> RecordWriter<T, W>
where
    T: GenericRecord,
    W: AsyncWriteExt + Unpin,
{
    /// Construct a [RecordWriter] from a writer with [AsyncWrite] trait.
    ///
    /// The constructed [RecordWriter] enables the asynchronous [send_async](RecordWriter::send_async) method.
    pub fn from_async_writer(writer: W) -> Result<Self> {
        Ok(Self {
            writer,
            _phantom: PhantomData,
        })
    }

    /// Write a record.
    pub async fn send_async(&mut self, record: T) -> Result<()> {
        let bytes = T::to_bytes(record)?;
        crate::io::r#async::try_write_record(&mut self.writer, bytes).await?;
        Ok(())
    }

    /// Flush the output stream asynchronously.
    pub async fn flush_async(&mut self) -> Result<()> {
        self.writer.flush().await?;
        Ok(())
    }

    pub fn into_sink(self) -> impl Sink<T, Error = Error> {
        sink::unfold(self, |mut writer, record| async move {
            writer.send_async(record).await?;
            Ok(writer)
        })
    }
}
