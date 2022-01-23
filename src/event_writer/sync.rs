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
    record_writer::RecordWriter,
};
use std::{
    borrow::Cow,
    convert::TryInto,
    fs,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    string::ToString,
};

/// The event writer.
///
/// It provies `write_scalar`, `write_image` methods, etc.
///
/// It can be built from a writer using [from_writer](EventWriter::from_writer), or write a new file
/// specified by path prefix using [from_writer](EventWriter::from_prefix).
///
/// ```rust
/// # async_std::task::block_on(async move {
/// use anyhow::Result;
/// use std::time::SystemTime;
/// use tch::{kind::FLOAT_CPU, Tensor};
/// use tfrecord::EventWriter;
///
/// let mut writer = EventWriter::from_prefix("log_dir/myprefix-", "", Default::default()).unwrap();
///
/// // step = 0, scalar = 3.14
/// writer.write_scalar("my_scalar", 0, 3.14)?;
///
/// // step = 1, specified wall time, histogram of [1, 2, 3, 4]
/// writer.write_histogram("my_histogram", (1, SystemTime::now()), vec![1, 2, 3, 4])?;
///
/// // step = 2, specified raw UNIX time in nanoseconds, random tensor of shape [8, 3, 16, 16]
/// writer.write_tensor(
///     "my_tensor",
///     (2, 1.594449514712264e+18),
///     Tensor::randn(&[8, 3, 16, 16], FLOAT_CPU),
/// )?;
/// # anyhow::Ok(())
/// # }).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EventWriter<W> {
    auto_flush: bool,
    events_writer: RecordWriter<Event, W>,
}

impl EventWriter<BufWriter<File>> {
    /// Build a writer writing events to a file.
    pub fn create<P>(path: P, config: EventWriterConfig) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path)?);
        Self::from_writer(writer, config)
    }

    /// Build a writer writing events to a file, which path is specified by a path prefix and file name suffix.
    pub fn from_prefix<'a, 'b, P, S>(
        prefix: P,
        file_name_suffix: S,
        config: EventWriterConfig,
    ) -> Result<EventWriter<BufWriter<File>>>
    where
        P: Into<Cow<'a, str>>,
        S: Into<Cow<'b, str>>,
    {
        let (dir_prefix, file_name) = super::create_tf_style_path(prefix, file_name_suffix)?;
        fs::create_dir_all(&dir_prefix)?;
        let path = dir_prefix.join(file_name);
        Self::create(path, config)
    }
}

impl<W> EventWriter<W>
where
    W: Write,
{
    /// Build from a writer with [Write] trait.
    pub fn from_writer(writer: W, config: EventWriterConfig) -> Result<Self>
    where
        W: Write,
    {
        let EventWriterConfig { auto_flush } = config;

        Ok(Self {
            auto_flush,
            events_writer: RecordWriter::from_writer(writer)?,
        })
    }

    /// Write a scalar summary.
    pub fn write_scalar(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        value: f32,
    ) -> Result<()> {
        let summary = Summary::from_scalar(tag, value)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Write a histogram summary.
    pub fn write_histogram(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        histogram: impl IntoHistogram,
    ) -> Result<()> {
        let summary = Summary::from_histogram(tag, histogram)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Write a tensor summary.
    pub fn write_tensor(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        tensor: impl TryInto<TensorProto, Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_tensor(tag, tensor)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Write an image summary.
    pub fn write_image(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        image: impl TryInto<Image, Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_image(tag, image)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Write a summary with multiple images.
    pub fn write_image_list(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        images: impl TryInfoImageList<Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_image_list(tag, images)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Write an audio summary.
    pub fn write_audio(
        &mut self,
        tag: impl ToString,
        event_meta: impl Into<EventMeta>,
        audio: impl TryInto<Audio, Error = impl Into<Error>>,
    ) -> Result<()> {
        let summary = Summary::from_audio(tag, audio)?;
        let event = event_meta.into().build_with_summary(summary);
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    // pub fn write_graph<>(&mut self, tag: impl ToString, event_meta: EventMeta) -> Result<(), Error>
    //
    // {
    //     todo!();
    // }

    /// Write a custom event.
    pub fn write_event(&mut self, event: Event) -> Result<()> {
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Flush this output stream.
    pub fn flush(&mut self) -> Result<()> {
        self.events_writer.flush()?;
        Ok(())
    }
}
