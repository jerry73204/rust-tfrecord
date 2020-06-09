use super::*;

/// The event writer initializer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventWriterInit;

impl EventWriterInit {
    /// Construct an [EventWriter] from a type with [Write] trait.
    pub fn from_writer<W>(writer: W) -> Result<EventWriter<W>, Error>
    where
        W: Write,
    {
        Ok(EventWriter {
            events_writer: RecordWriterInit::from_writer(writer)?,
        })
    }

    /// Construct an [EventWriter] by creating a file at specified path.
    pub fn create<P>(path: P) -> Result<EventWriter<std::io::BufWriter<std::fs::File>>, Error>
    where
        P: AsRef<Path>,
    {
        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        Self::from_writer(writer)
    }

    /// Construct an [EventWriter] with TensorFlow-style path prefix and an optional file name suffix.
    pub fn from_prefix<S1>(
        prefix: S1,
        file_name_suffix: Option<String>,
    ) -> Result<EventWriter<std::io::BufWriter<std::fs::File>>, Error>
    where
        S1: AsRef<str>,
    {
        let (dir_prefix, file_name) = Self::create_tf_style_path(prefix, file_name_suffix)?;
        fs::create_dir_all(&dir_prefix)?;
        let path = dir_prefix.join(file_name);
        Self::create(path)
    }

    /// Construct an [EventWriter] from a type with [AsyncWriteExt] trait.
    #[cfg(feature = "async_")]
    pub fn from_async_writer<W>(writer: W) -> Result<EventWriter<W>, Error>
    where
        W: AsyncWriteExt,
    {
        Ok(EventWriter {
            events_writer: RecordWriterInit::from_async_writer(writer)?,
        })
    }

    /// Construct an [EventWriter] by creating a file at specified path.
    #[cfg(feature = "async_")]
    pub async fn create_async<P>(
        path: P,
    ) -> Result<EventWriter<async_std::io::BufWriter<async_std::fs::File>>, Error>
    where
        P: AsRef<async_std::path::Path>,
    {
        let writer = async_std::io::BufWriter::new(async_std::fs::File::create(path).await?);
        Self::from_async_writer(writer)
    }

    /// Construct an asynchronous [EventWriter] with TensorFlow-style path prefix and an optional file name suffix.
    #[cfg(feature = "async_")]
    pub async fn from_prefix_async<S1>(
        prefix: S1,
        file_name_suffix: Option<String>,
    ) -> Result<EventWriter<async_std::io::BufWriter<async_std::fs::File>>, Error>
    where
        S1: AsRef<str>,
    {
        let (dir_prefix, file_name) = Self::create_tf_style_path(prefix, file_name_suffix)?;
        async_std::fs::create_dir_all(&dir_prefix).await?;
        let path = dir_prefix.join(file_name);
        Self::create_async(path).await
    }

    fn create_tf_style_path<S1>(
        prefix: S1,
        file_name_suffix: Option<String>,
    ) -> Result<(PathBuf, String), Error>
    where
        S1: AsRef<str>,
    {
        let file_name_suffix = file_name_suffix
            .map(|suffix| suffix.to_string())
            .unwrap_or("".into());
        let prefix = {
            let prefix = prefix.as_ref();
            if prefix.is_empty() {
                return Err(Error::InvalidArgumentsError {
                    desc: "the prefix must not be empty".into(),
                });
            }
            prefix
        };

        let (dir_prefix, file_name_prefix): (PathBuf, String) = match prefix.chars().last() {
            Some(MAIN_SEPARATOR) => {
                let dir_prefix = PathBuf::from(prefix);
                let file_name_prefix = "".into();
                (dir_prefix, file_name_prefix)
            }
            _ => {
                let path = PathBuf::from(prefix);
                let file_name_prefix = match path.file_name() {
                    Some(file_name) => file_name
                        .to_str()
                        .ok_or_else(|| Error::UnicodeError {
                            desc: format!("the path {} is not unicode", path.display()),
                        })?
                        .to_string(),
                    None => "".into(),
                };
                let dir_prefix = path.parent().map(ToOwned::to_owned).unwrap_or(path);
                (dir_prefix, file_name_prefix)
            }
        };

        let file_name = {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            let host_name = hostname::get()?
                .into_string()
                .map_err(|_| Error::UnicodeError {
                    desc: "the host name is not Unicode".into(),
                })?;
            let file_name = format!(
                "{}.out.tfevents.{}.{}{}",
                file_name_prefix, timestamp, host_name, file_name_suffix
            );
            file_name
        };

        Ok((dir_prefix, file_name))
    }
}

/// The event writer type.
///
/// It is a wrapper of [RecordWriter] with extra capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct EventWriter<W> {
    events_writer: RecordWriter<Event, W>,
}

impl<W> EventWriter<W>
where
    W: Write,
{
    /// Write a scalar summary.
    pub fn write_scalar<T>(
        &mut self,
        tag: T,
        event_init: EventInit,
        value: f32,
    ) -> Result<(), Error>
    where
        T: ToString,
    {
        let summary = SummaryInit { tag }.build_scalar(value)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send(event)?;
        self.events_writer.flush()?;
        Ok(())
    }

    /// Write a histogram summary.
    pub fn write_histogram<T, H>(
        &mut self,
        tag: T,
        event_init: EventInit,
        histogram: H,
    ) -> Result<(), Error>
    where
        T: ToString,
        H: TryInto<HistogramProto, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_histogram(histogram)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send(event)?;
        self.events_writer.flush()?;
        Ok(())
    }

    /// Write a tensor summary.
    pub fn write_tensor<T, S>(
        &mut self,
        tag: T,
        event_init: EventInit,
        tensor: S,
    ) -> Result<(), Error>
    where
        T: ToString,
        S: TryInto<TensorProto, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_tensor(tensor)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send(event)?;
        self.events_writer.flush()?;
        Ok(())
    }

    /// Write an image summary.
    pub fn write_image<T, M>(
        &mut self,
        tag: T,
        event_init: EventInit,
        image: M,
    ) -> Result<(), Error>
    where
        T: ToString,
        M: TryInto<Image, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_image(image)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send(event)?;
        self.events_writer.flush()?;
        Ok(())
    }

    /// Write an audio summary.
    pub fn write_audio<T, A>(
        &mut self,
        tag: T,
        event_init: EventInit,
        audio: A,
    ) -> Result<(), Error>
    where
        T: ToString,
        A: TryInto<Audio, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_audio(audio)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send(event)?;
        self.events_writer.flush()?;
        Ok(())
    }

    // pub fn write_graph<T>(&mut self, tag: T, event_init: EventInit) -> Result<(), Error>
    // where
    //     T: ToString,
    // {
    //     todo!();
    // }

    /// Write a custom event.
    pub fn write_event(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send(event)?;
        self.events_writer.flush()?;
        Ok(())
    }
}

#[cfg(feature = "async_")]
impl<W> EventWriter<W>
where
    W: AsyncWriteExt + Unpin,
{
    /// Write a scalar summary asynchronously.
    pub async fn write_scalar_async<T>(
        &mut self,
        tag: T,
        event_init: EventInit,
        value: f32,
    ) -> Result<(), Error>
    where
        T: ToString,
    {
        let summary = SummaryInit { tag }.build_scalar(value)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        self.events_writer.flush_async().await?;
        Ok(())
    }

    /// Write a histogram summary asynchronously.
    pub async fn write_histogram_async<T, H>(
        &mut self,
        tag: T,
        event_init: EventInit,
        histogram: H,
    ) -> Result<(), Error>
    where
        T: ToString,
        H: TryInto<HistogramProto, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_histogram(histogram)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        self.events_writer.flush_async().await?;
        Ok(())
    }

    /// Write a tensor summary asynchronously.
    pub async fn write_tensor_async<T, S>(
        &mut self,
        tag: T,
        event_init: EventInit,
        tensor: S,
    ) -> Result<(), Error>
    where
        T: ToString,
        S: TryInto<TensorProto, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_tensor(tensor)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        self.events_writer.flush_async().await?;
        Ok(())
    }

    /// Write an image summary asynchronously.
    pub async fn write_image_async<T, M>(
        &mut self,
        tag: T,
        event_init: EventInit,
        image: M,
    ) -> Result<(), Error>
    where
        T: ToString,
        M: TryInto<Image, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_image(image)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        self.events_writer.flush_async().await?;
        Ok(())
    }

    /// Write an audio summary asynchronously.
    pub async fn write_audio_async<T, A>(
        &mut self,
        tag: T,
        event_init: EventInit,
        audio: A,
    ) -> Result<(), Error>
    where
        T: ToString,
        A: TryInto<Audio, Error = Error>,
    {
        let summary = SummaryInit { tag }.build_audio(audio)?;
        let event = event_init.build_with_summary(summary);
        self.events_writer.send_async(event).await?;
        self.events_writer.flush_async().await?;
        Ok(())
    }

    // pub async fn write_graph<T>(&mut self, tag: T, event_init: EventInit) -> Result<(), Error>
    // where
    //     T: ToString,
    // {
    //     todo!();
    // }

    /// Write a custom event asynchronously.
    pub async fn write_event_async(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send_async(event).await?;
        self.events_writer.flush_async().await?;
        Ok(())
    }
}
