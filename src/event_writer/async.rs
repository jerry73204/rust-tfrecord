use super::EventWriterConfig;
use crate::{
    error::{Error, Result},
    event::EventMeta,
    markers::TryInfoImageList,
    protobuf::{
        summary::{Audio, Image},
        Event, Summary, TensorProto,
    },
    protobuf_ext::IntoHistogram,
    record_writer::RecordAsyncWriter,
};
use async_std::{fs::File, io::BufWriter, path::Path};
use futures::io::AsyncWrite;
use std::{borrow::Cow, convert::TryInto, string::ToString};

/// The event writer.
///
/// It provies `write_scalar`, `write_image` methods, etc.
///
/// It can be built from a writer using [from_writer](EventAsyncWriter::from_writer), or write a new file
/// specified by path prefix using [from_writer](EventAsyncWriter::from_prefix).
///
/// ```rust
/// # async_std::task::block_on(async move {
/// use anyhow::Result;
/// use std::time::SystemTime;
/// use tch::{kind::FLOAT_CPU, Tensor};
/// use tfrecord::EventAsyncWriter;
///
/// let mut writer = EventAsyncWriter::from_prefix("log_dir/myprefix-", "", Default::default())
///     .await
///     .unwrap();
///
/// // step = 0, scalar = 3.14
/// writer.write_scalar("my_scalar", 0, 3.14).await?;
///
/// // step = 1, specified wall time, histogram of [1, 2, 3, 4]
/// writer
///     .write_histogram("my_histogram", (1, SystemTime::now()), vec![1, 2, 3, 4])
///     .await?;
///
/// // step = 2, specified raw UNIX time in nanoseconds, random tensor of shape [8, 3, 16, 16]
/// writer
///     .write_tensor(
///         "my_tensor",
///         (2, 1.594449514712264e+18),
///         Tensor::randn(&[8, 3, 16, 16], FLOAT_CPU),
///     )
///     .await?;
/// # anyhow::Ok(())
/// # }).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EventAsyncWriter<W> {
    auto_flush: bool,
    events_writer: RecordAsyncWriter<Event, W>,
}

impl EventAsyncWriter<BufWriter<File>> {
    /// Build a writer writing events to a file.
    pub async fn create<P>(path: P, config: EventWriterConfig) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path).await?);
        Self::from_writer(writer, config)
    }

    /// Build a writer writing events to a file, which path is specified by a path prefix and file name suffix.
    pub async fn from_prefix<'a, 'b, P, S>(
        prefix: P,
        file_name_suffix: S,
        config: EventWriterConfig,
    ) -> Result<Self>
    where
        P: Into<Cow<'a, str>>,
        S: Into<Cow<'b, str>>,
    {
        let (dir_prefix, file_name) = super::create_tf_style_path(prefix, file_name_suffix)?;
        async_std::fs::create_dir_all(&dir_prefix).await?;
        let path = dir_prefix.join(file_name);
        Self::create(path, config).await
    }
}

impl<W> EventAsyncWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Build from a writer with [AsyncWrite] trait.
    pub fn from_writer(writer: W, config: EventWriterConfig) -> Result<Self> {
        let EventWriterConfig { auto_flush } = config;
        Ok(Self {
            auto_flush,
            events_writer: RecordAsyncWriter::from_writer(writer)?,
        })
    }

    /// Write a scalar summary asynchronously.
    pub async fn write_scalar(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        value: f32,
    ) -> Result<()> {
        let summary = Summary::from_scalar(tag, value)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Write a histogram summary asynchronously.
    pub async fn write_histogram(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        histogram: impl IntoHistogram,
    ) -> Result<()> {
        let summary = Summary::from_histogram(tag, histogram)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Write a tensor summary asynchronously.
    pub async fn write_tensor(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        tensor: impl TryInto<TensorProto, Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_tensor(tag, tensor)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Write an image summary asynchronously.
    pub async fn write_image(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        image: impl TryInto<Image, Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_image(tag, image)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Write a summary with multiple images asynchronously.
    pub async fn write_image_list(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        images: impl TryInfoImageList<Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_image_list(tag, images)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Write an audio summary asynchronously.
    pub async fn write_audio(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        audio: impl TryInto<Audio, Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_audio(tag, audio)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Write a custom event asynchronously.
    pub async fn write_event(&mut self, event: Event) -> Result<()> {
        self.events_writer.send(event).await?;
        if self.auto_flush {
            self.events_writer.flush().await?;
        }
        Ok(())
    }

    /// Flush this output stream asynchronously.
    pub async fn flush(&mut self) -> Result<()> {
        self.events_writer.flush().await?;
        Ok(())
    }
}
