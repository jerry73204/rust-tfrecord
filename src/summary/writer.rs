use super::*;

/// The writer initializer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventWriterInit;

impl EventWriterInit {
    /// Construct a [EventWriter] from a type with [Write] trait.
    pub fn from_writer<W>(writer: W) -> Result<EventWriter<W>, Error>
    where
        W: Write,
    {
        Ok(EventWriter {
            events_writer: RecordWriterInit::from_writer(writer)?,
        })
    }

    /// Construct a [EventWriter] by creating a file at specified path.
    pub fn create<P>(path: P) -> Result<EventWriter<std::io::BufWriter<std::fs::File>>, Error>
    where
        P: AsRef<Path>,
    {
        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        Self::from_writer(writer)
    }

    /// Construct a [EventWriter] from a type with [AsyncWrite] trait.
    #[cfg(feature = "async_")]
    pub fn from_async_writer<W>(writer: W) -> Result<EventWriter<W>, Error>
    where
        W: AsyncWrite,
    {
        Ok(EventWriter {
            events_writer: RecordWriterInit::from_async_writer(writer)?,
        })
    }

    /// Construct a [EventWriter] by creating a file at specified path.
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventWriter<W> {
    events_writer: RecordWriter<Event, W>,
}

impl<W> EventWriter<W>
where
    W: Write,
{
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
        Ok(())
    }

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
        Ok(())
    }

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
        Ok(())
    }

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
        Ok(())
    }

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
        Ok(())
    }

    // pub fn write_graph<T>(&mut self, tag: T, event_init: EventInit) -> Result<(), Error>
    // where
    //     T: ToString,
    // {
    //     todo!();
    // }

    pub fn write_event(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send(event)
    }
}

#[cfg(feature = "async_")]
impl<W> EventWriter<W>
where
    W: AsyncWrite + Unpin,
{
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
        Ok(())
    }

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
        Ok(())
    }

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
        Ok(())
    }

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
        Ok(())
    }

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
        Ok(())
    }

    // pub async fn write_graph<T>(&mut self, tag: T, event_init: EventInit) -> Result<(), Error>
    // where
    //     T: ToString,
    // {
    //     todo!();
    // }

    pub async fn write_event_async(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send_async(event).await
    }
}
