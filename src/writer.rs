//! Writing TFRecord data format.
//!
//! The [RecordWriter] is initialized by [RecordWriterInit]. It can write
//! either [Example], [RawExampple](crate::RawExample), [Vec\<u8\>](Vec), and many other record types.
//! that implements [GenericRecord], depending on your choice.
//!
//! The type aliases [ExampleWriter], [RawExampleWriter] and [BytesWriter]
//! are [RecordWriter] writing specific record types.

use crate::{error::Error, markers::GenericRecord, protos::Example as RawExample, types::Example};
#[cfg(feature = "async_")]
use futures::io::AsyncWriteExt;
use std::{io::Write, marker::PhantomData, path::Path};

/// Alias to [RecordWriter] which input record type is [Vec<u8>](Vec).
pub type BytesWriter<W> = RecordWriter<Vec<u8>, W>;
/// Alias to [RecordWriter] which input record type is [RawExample].
pub type RawExampleWriter<W> = RecordWriter<RawExample, W>;
/// Alias to [RecordWriter] which input record type is [Example].
pub type ExampleWriter<W> = RecordWriter<Example, W>;

/// The writer initializer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordWriterInit;

impl RecordWriterInit {
    /// Construct a [RecordWriter] from a type with [Write] trait.
    ///
    /// The constructed [RecordWriter] enables the blocking [send](RecordWriter::send) method.
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

    /// Construct a [RecordWriter] by creating a file at specified path.
    ///
    /// The constructed [RecordWriter] enables the blocking [send](RecordWriter::send) method.
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

    /// Construct a [RecordWriter] from a type with [AsyncWriteExt] trait.
    ///
    /// The constructed [RecordWriter] enables the asynchronous [send_async](RecordWriter::send_async) method.
    #[cfg(feature = "async_")]
    pub fn from_async_writer<T, W>(writer: W) -> Result<RecordWriter<T, W>, Error>
    where
        T: GenericRecord,
        W: AsyncWriteExt,
    {
        Ok(RecordWriter {
            writer,
            _phantom: PhantomData,
        })
    }

    /// Construct a [RecordWriter] by creating a file at specified path.
    ///
    /// The constructed [RecordWriter] enables the asynchronous [send_async](RecordWriter::send_async) method.
    #[cfg(feature = "async_")]
    pub async fn create_async<T, P>(
        path: P,
    ) -> Result<RecordWriter<T, async_std::io::BufWriter<async_std::fs::File>>, Error>
    where
        T: GenericRecord,
        P: AsRef<async_std::path::Path>,
    {
        let writer = async_std::io::BufWriter::new(async_std::fs::File::create(path).await?);
        Self::from_async_writer(writer)
    }
}

/// The writer type.
///
/// It provides blocing [RecordWriter::send] and analogues [RecordWriter::send_async] methods
/// to write records.

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
    /// Write a record.
    ///
    /// The method is enabled if the underlying writer implements [Write].
    pub fn send(&mut self, record: T) -> Result<(), Error> {
        let bytes = T::to_bytes(record)?;
        crate::io::blocking::try_write_record(&mut self.writer, bytes)?;
        Ok(())
    }

    /// Flush the output stream.
    pub fn flush(&mut self) -> Result<(), Error> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(feature = "async_")]
impl<T, W> RecordWriter<T, W>
where
    T: GenericRecord,
    W: AsyncWriteExt + Unpin,
{
    /// Write a record.
    ///
    /// The method is enabled if the underlying writer implements [AsyncWriteExt].
    pub async fn send_async(&mut self, record: T) -> Result<(), Error> {
        let bytes = T::to_bytes(record)?;
        crate::io::async_::try_write_record(&mut self.writer, bytes).await?;
        Ok(())
    }

    /// Flush the output stream asynchronously.
    pub async fn flush_async(&mut self) -> Result<(), Error> {
        self.writer.flush().await?;
        Ok(())
    }
}
