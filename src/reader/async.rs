use super::RecordReaderConfig;
use crate::{
    error::Error,
    markers::GenericRecord,
    protobuf::{Event, Example as RawExample},
    types::Example,
};
use async_std::{fs::File, io::BufReader, path::Path};
use futures::{
    io::AsyncRead,
    stream::{BoxStream, Stream, StreamExt},
};
use pin_project::pin_project;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub type BytesStream<R> = RecordStream<Vec<u8>, R>;

pub type RawExampleStream<R> = RecordStream<RawExample, R>;

pub type ExampleStream<R> = RecordStream<Example, R>;

pub type EventStream<R> = RecordStream<Event, R>;

#[pin_project]
pub struct RecordStream<T, R>
where
    T: GenericRecord,
    R: AsyncRead,
{
    #[pin]
    stream: BoxStream<'static, Result<T, Error>>,
    _phantom: PhantomData<R>,
}

impl<T, R> RecordStream<T, R>
where
    T: GenericRecord,
    R: AsyncRead,
{
    /// Build a stream from a reader type with [AsyncRead] trait.
    ///
    /// Specify the output type while calling this method. For example,
    /// `from_reader<Example, _>()`, or you can use [bytes_from_reader](RecordStreamInit::bytes_from_reader),
    /// [raw_examples_from_reader](RecordStreamInit::raw_examples_from_reader) and
    /// [examples_from_reader](RecordStreamInit::examples_from_reader) aliases.
    pub fn from_reader(reader: R, config: RecordReaderConfig) -> Self
    where
        R: 'static + Unpin + Send,
    {
        let RecordReaderConfig { check_integrity } = config;

        let stream = futures::stream::try_unfold(reader, move |mut reader| async move {
            let bytes = crate::io::r#async::try_read_record(&mut reader, check_integrity).await?;
            let record = bytes.map(T::from_bytes).transpose()?;
            Ok(record.map(|record| (record, reader)))
        })
        .boxed();

        Self {
            stream,
            _phantom: PhantomData,
        }
    }
}

impl<T> RecordStream<T, BufReader<File>>
where
    T: GenericRecord,
{
    /// Build a stream from a path.
    ///
    /// Specify the output type while calling this method. For example,
    /// `open<Example, _>()`, or you can use [bytes_open](RecordStreamInit::bytes_open),
    /// [raw_examples_open](RecordStreamInit::raw_examples_open) and
    /// [examples_open](RecordStreamInit::examples_open) aliases.
    pub async fn open<P>(path: P, config: RecordReaderConfig) -> Result<Self, Error>
    where
        T: GenericRecord,
        P: AsRef<Path>,
    {
        let reader = BufReader::new(File::open(path).await?);
        let reader = Self::from_reader(reader, config);
        Ok(reader)
    }
}

impl<T, R> Stream for RecordStream<T, R>
where
    T: GenericRecord,
    R: AsyncRead,
{
    type Item = Result<T, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}
