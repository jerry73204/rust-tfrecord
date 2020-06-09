use super::*;

/// A [Event] initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct EventInit {
    /// The wall clock time in microseconds.
    ///
    /// If the field is set to `None`, it sets to current system time when the event is built.
    pub wall_time: Option<f64>,
    /// The global step.
    pub step: i64,
}

impl EventInit {
    /// Create a initializer with global step and wall time.
    pub fn new(step: i64, wall_time: f64) -> Self {
        Self {
            wall_time: Some(wall_time),
            step,
        }
    }

    /// Create a initializer with global step and without wall time.
    pub fn with_step(step: i64) -> Self {
        Self {
            wall_time: None,
            step,
        }
    }

    /// Build an empty event.
    pub fn build_empty(self) -> Event {
        let (wall_time, step) = self.to_parts();
        Event {
            wall_time,
            step,
            what: None,
        }
    }

    /// Build an event with summary.
    pub fn build_with_summary(self, summary: Summary) -> Event {
        let (wall_time, step) = self.to_parts();
        Event {
            wall_time,
            step,
            what: Some(What::Summary(summary)),
        }
    }

    fn to_parts(self) -> (f64, i64) {
        let Self {
            wall_time: wall_time_opt,
            step,
        } = self;
        let wall_time = wall_time_opt.unwrap_or_else(|| Self::get_wall_time());
        (wall_time, step)
    }

    fn get_wall_time() -> f64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f64
            / 1.0e9
    }
}

/// A [Summary] initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct SummaryInit<T>
where
    T: ToString,
{
    /// The tag of the summary.
    pub tag: T,
}

impl<T> SummaryInit<T>
where
    T: ToString,
{
    /// Create an initializer with a tag.
    pub fn new(tag: T) -> Self {
        Self { tag }
    }

    /// Build a scalar summary.
    pub fn build_scalar(self, value: f32) -> Result<Summary, Error> {
        let Self { tag } = self;

        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::SimpleValue(value)),
            }],
        };
        Ok(summary)
    }

    /// Build a histogram summary.
    pub fn build_histogram<H>(self, histogram: H) -> Result<Summary, Error>
    where
        H: TryInto<HistogramProto, Error = Error>,
    {
        let Self { tag } = self;

        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Histo(histogram.try_into()?)),
            }],
        };
        Ok(summary)
    }

    /// Build a tensor summary.
    pub fn build_tensor<S>(self, tensor: S) -> Result<Summary, Error>
    where
        S: TryInto<TensorProto, Error = Error>,
    {
        let Self { tag } = self;

        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Tensor(tensor.try_into()?)),
            }],
        };
        Ok(summary)
    }

    /// Build an image summary.
    pub fn build_image<M>(self, image: M) -> Result<Summary, Error>
    where
        M: TryInto<Image, Error = Error>,
    {
        let Self { tag } = self;

        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Image(image.try_into()?)),
            }],
        };
        Ok(summary)
    }

    /// Build an audio summary.
    pub fn build_audio<A>(self, audio: A) -> Result<Summary, Error>
    where
        A: TryInto<Audio, Error = Error>,
    {
        let Self { tag } = self;

        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(ValueEnum::Audio(audio.try_into()?)),
            }],
        };
        Ok(summary)
    }
}
