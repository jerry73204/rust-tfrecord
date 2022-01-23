use super::{EventWriter, EventWriterConfig};
use crate::{
    error::Error,
    markers::TryInfoImageList,
    protobuf::{
        summary::{Audio, Image},
        Event, Summary, TensorProto,
    },
    protobuf_ext::IntoHistogram,
    summary::event::EventMeta,
    writer::RecordWriter,
};
use async_std::{fs::File, io::BufWriter, path::Path};
use futures::io::AsyncWrite;
use std::{borrow::Cow, convert::TryInto, string::ToString};

impl EventWriter<BufWriter<File>> {
    /// Construct an [EventWriter] by creating a file at specified path.
    pub async fn create_async<P>(path: P, config: EventWriterConfig) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path).await?);
        Self::from_async_writer(writer, config)
    }

    /// Construct an asynchronous [EventWriter] with TensorFlow-style path prefix and an optional file name suffix.
    pub async fn from_prefix_async<'a, 'b, P, S>(
        prefix: P,
        file_name_suffix: S,
        config: EventWriterConfig,
    ) -> Result<Self, Error>
    where
        P: Into<Cow<'a, str>>,
        S: Into<Cow<'b, str>>,
    {
        let (dir_prefix, file_name) = super::create_tf_style_path(prefix, file_name_suffix)?;
        async_std::fs::create_dir_all(&dir_prefix).await?;
        let path = dir_prefix.join(file_name);
        Self::create_async(path, config).await
    }
}

impl<W> EventWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Construct an [EventWriter] from a type with [AsyncWriteExt] trait.
    pub fn from_async_writer(writer: W, config: EventWriterConfig) -> Result<Self, Error> {
        let EventWriterConfig { auto_flush } = config;
        Ok(Self {
            auto_flush,
            events_writer: RecordWriter::from_async_writer(writer)?,
        })
    }

    /// Write a scalar summary asynchronously.
    pub async fn write_scalar_async(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        value: f32,
    ) -> Result<(), Error> {
        let summary = Summary::from_scalar(tag, value)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Write a histogram summary asynchronously.
    pub async fn write_histogram_async(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        histogram: impl IntoHistogram,
    ) -> Result<(), Error> {
        let summary = Summary::from_histogram(tag, histogram)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Write a tensor summary asynchronously.
    pub async fn write_tensor_async(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        tensor: impl TryInto<TensorProto, Error = impl Into<Error>>,
    ) -> Result<(), Error> {
        let summary = Summary::from_tensor(tag, tensor)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Write an image summary asynchronously.
    pub async fn write_image_async(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        image: impl TryInto<Image, Error = impl Into<Error>>,
    ) -> Result<(), Error> {
        let summary = Summary::from_image(tag, image)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Write a summary with multiple images asynchronously.
    pub async fn write_image_list_async(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        images: impl TryInfoImageList<Error = impl Into<Error>>,
    ) -> Result<(), Error> {
        let summary = Summary::from_image_list(tag, images)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Write an audio summary asynchronously.
    pub async fn write_audio_async(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        audio: impl TryInto<Audio, Error = impl Into<Error>>,
    ) -> Result<(), Error> {
        let summary = Summary::from_audio(tag, audio)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Write a custom event asynchronously.
    pub async fn write_event_async(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send_async(event).await?;
        if self.auto_flush {
            self.events_writer.flush_async().await?;
        }
        Ok(())
    }

    /// Flush this output stream asynchronously.
    pub async fn flush_async(&mut self) -> Result<(), Error> {
        self.events_writer.flush_async().await?;
        Ok(())
    }
}
