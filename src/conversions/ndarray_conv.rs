#![cfg(feature = "with-ndarray")]

use super::*;
use ndarray::{ArrayBase, Data, Dimension, RawData};

// to histogram

impl<S, D> TryFrom<&ArrayBase<S, D>> for HistogramProto
where
    S: RawData<Elem = f64> + Data,
    D: Dimension,
{
    type Error = Error;

    fn try_from(from: &ArrayBase<S, D>) -> Result<Self, Self::Error> {
        let histogram = Histogram::default();
        let values_iter = from.iter().cloned().map(|value| {
            R64::try_new(value).ok_or_else(|| Error::ConversionError {
                desc: "non-finite floating value found".into(),
            })
        });

        for result in values_iter {
            let value = result?;
            histogram.add(value);
        }

        Ok(histogram.into())
    }
}

impl<S, D> TryFrom<ArrayBase<S, D>> for HistogramProto
where
    S: RawData<Elem = f64> + Data,
    D: Dimension,
{
    type Error = Error;

    fn try_from(from: ArrayBase<S, D>) -> Result<Self, Self::Error> {
        Self::try_from(&from)
    }
}

// array to tensor

impl<S, D, T> From<&ArrayBase<S, D>> for TensorProto
where
    D: Dimension,
    S: RawData<Elem = T> + Data,
    T: TensorProtoElement,
{
    fn from(from: &ArrayBase<S, D>) -> Self {
        TensorProtoInit {
            shape: Some(from.shape()),
        }
        .build_with_data(&from.iter().cloned().collect::<Vec<T>>())
    }
}

impl<S, D, T> From<ArrayBase<S, D>> for TensorProto
where
    D: Dimension,
    S: RawData<Elem = T> + Data,
    T: TensorProtoElement,
{
    fn from(from: ArrayBase<S, D>) -> Self {
        Self::from(&from)
    }
}
