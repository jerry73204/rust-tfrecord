use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SummaryWriter<W> {
    events_writer: RecordWriter<Event, W>,
}

impl<W> SummaryWriter<W>
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
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::SimpleValue(value)),
            }],
        };
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
        H: Into<HistogramProto>,
    {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Histo(histogram.into())),
            }],
        };
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
        S: Into<TensorProto>,
    {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Tensor(tensor.into())),
            }],
        };
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
        M: Into<Image>,
    {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Image(image.into())),
            }],
        };
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
        A: Into<Audio>,
    {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Audio(audio.into())),
            }],
        };
        let event = event_init.build_with_summary(summary);
        self.events_writer.send(event)?;
        Ok(())
    }

    pub fn write_graph<T>(&mut self, tag: T, event_init: EventInit) -> Result<(), Error>
    where
        T: ToString,
    {
        todo!();
    }

    pub fn write_event(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send(event)
    }
}

#[cfg(feature = "async_")]
impl<W> SummaryWriter<W>
where
    W: AsyncWrite + Unpin,
{
    pub async fn write_scalar_async(&mut self, event_init: EventInit) -> Result<(), Error> {
        todo!();
    }

    pub async fn write_tensor_async(&mut self, event_init: EventInit) -> Result<(), Error> {
        todo!();
    }

    pub async fn write_histogram_async(&mut self, event_init: EventInit) -> Result<(), Error> {
        todo!();
    }

    pub async fn write_image_async(&mut self, event_init: EventInit) -> Result<(), Error> {
        todo!();
    }

    pub async fn write_audio_async(&mut self, event_init: EventInit) -> Result<(), Error> {
        todo!();
    }

    pub async fn write_graph_async(&mut self, event_init: EventInit) -> Result<(), Error> {
        todo!();
    }

    pub async fn write_event_async(&mut self, event: Event) -> Result<(), Error> {
        self.events_writer.send_async(event).await
    }
}
