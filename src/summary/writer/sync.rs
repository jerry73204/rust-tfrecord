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
use std::{
    borrow::Cow,
    convert::TryInto,
    fs,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    string::ToString,
};

impl EventWriter<BufWriter<File>> {
    /// Construct an [EventWriter] by creating a file at specified path.
    pub fn create<P>(path: P, config: EventWriterConfig) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path)?);
        Self::from_writer(writer, config)
    }

    /// Construct an [EventWriter] with TensorFlow-style path prefix and an optional file name suffix.
    pub fn from_prefix<'a, 'b, P, S>(
        prefix: P,
        file_name_suffix: S,
        config: EventWriterConfig,
    ) -> Result<EventWriter<BufWriter<File>>, Error>
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
    /// Construct an [EventWriter] from a type with [Write] trait.
    pub fn from_writer(writer: W, config: EventWriterConfig) -> Result<Self, Error>
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
    pub fn write_event(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send(event)?;
        if self.auto_flush {
            self.events_writer.flush()?;
        }
        Ok(())
    }

    /// Flush this output stream.
    pub fn flush(&mut self) -> Result<(), Error> {
        self.events_writer.flush()?;
        Ok(())
    }
}
