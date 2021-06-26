use super::*;

// protobuf Feature from/to crate's Feature

impl From<RawFeature> for Feature {
    fn from(from: RawFeature) -> Self {
        match from.kind {
            Some(Kind::BytesList(BytesList { value })) => Feature::BytesList(value),
            Some(Kind::FloatList(FloatList { value })) => Feature::FloatList(value),
            Some(Kind::Int64List(Int64List { value })) => Feature::Int64List(value),
            None => Feature::None,
        }
    }
}

impl From<&RawFeature> for Feature {
    fn from(from: &RawFeature) -> Self {
        Self::from(from.to_owned())
    }
}

impl From<Feature> for RawFeature {
    fn from(from: Feature) -> Self {
        let kind = match from {
            Feature::BytesList(value) => Some(Kind::BytesList(BytesList { value })),
            Feature::FloatList(value) => Some(Kind::FloatList(FloatList { value })),
            Feature::Int64List(value) => Some(Kind::Int64List(Int64List { value })),
            Feature::None => None,
        };
        Self { kind }
    }
}

impl From<&Feature> for RawFeature {
    fn from(from: &Feature) -> Self {
        Self::from(from.to_owned())
    }
}

// protobuf Example from/to crate's Example

impl From<RawExample> for Example {
    fn from(from: RawExample) -> Self {
        let features = match from.features {
            Some(features) => features,
            None => return HashMap::new(),
        };
        features
            .feature
            .into_iter()
            .map(|(name, feature)| (name, Feature::from(feature)))
            .collect::<HashMap<_, _>>()
    }
}

impl From<&RawExample> for Example {
    fn from(from: &RawExample) -> Self {
        Self::from(from.to_owned())
    }
}

impl From<Example> for RawExample {
    fn from(from: Example) -> Self {
        let feature = from
            .into_iter()
            .map(|(name, feature)| (name, RawFeature::from(feature)))
            .collect::<HashMap<_, _>>();
        if feature.is_empty() {
            RawExample { features: None }
        } else {
            RawExample {
                features: Some(Features { feature }),
            }
        }
    }
}

impl From<&Example> for RawExample {
    fn from(from: &Example) -> Self {
        Self::from(from.to_owned())
    }
}

// built-in Histogram to HistogramProto

impl From<Histogram> for HistogramProto {
    fn from(from: Histogram) -> Self {
        let state = from.0.read().unwrap();

        let counts: Vec<_> = state
            .buckets
            .iter()
            .map(|bucket| bucket.count.load(Ordering::Relaxed) as f64)
            .collect();
        let limits: Vec<_> = state
            .buckets
            .iter()
            .map(|bucket| bucket.limit.raw())
            .collect();

        let min = state
            .stat
            .as_ref()
            .map(|stat| stat.min)
            .unwrap_or(f64::INFINITY);
        let max = state
            .stat
            .as_ref()
            .map(|stat| stat.max)
            .unwrap_or(f64::NEG_INFINITY);
        let sum = state.stat.as_ref().map(|stat| stat.sum).unwrap_or(0.0);
        let sum_squares = state
            .stat
            .as_ref()
            .map(|stat| stat.sum_squares)
            .unwrap_or(0.0);
        let len = state.stat.as_ref().map(|stat| stat.len).unwrap_or(0);

        Self {
            min,
            max,
            num: len as f64,
            sum,
            sum_squares,
            bucket_limit: limits,
            bucket: counts,
        }
    }
}

// slice or vec to histogram

impl<T> From<&[T]> for HistogramProto
where
    T: HistogramProtoElement,
{
    fn from(from: &[T]) -> Self {
        let histogram = Histogram::default();
        from.iter()
            .cloned()
            .for_each(|value| histogram.add(R64::new(value.to_f64())));
        histogram.into()
    }
}

impl<T> From<&Vec<T>> for HistogramProto
where
    T: HistogramProtoElement,
{
    fn from(from: &Vec<T>) -> Self {
        Self::from(from.as_slice())
    }
}

impl<T> From<Vec<T>> for HistogramProto
where
    T: HistogramProtoElement,
{
    fn from(from: Vec<T>) -> Self {
        Self::from(from.as_slice())
    }
}

// slice or vec to TensorProto

impl<S> From<&[S]> for TensorProto
where
    S: AsRef<[u8]>,
{
    fn from(from: &[S]) -> Self {
        TensorProtoInit {
            shape: Some(vec![from.len()]),
        }
        .build_string(from)
    }
}

impl<S> From<&Vec<S>> for TensorProto
where
    S: AsRef<[u8]>,
{
    fn from(from: &Vec<S>) -> Self {
        From::<&[_]>::from(from.as_ref())
    }
}

impl<S> From<Vec<S>> for TensorProto
where
    S: AsRef<[u8]>,
{
    fn from(from: Vec<S>) -> Self {
        From::<&[_]>::from(from.as_ref())
    }
}
