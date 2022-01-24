use super::IntoHistogram;
use crate::{
    error::Error,
    protobuf::{
        summary::{value, Audio, Image, Value},
        Summary, TensorProto,
    },
    protobuf_ext::IntoImageList,
};

impl Summary {
    /// Build a scalar summary.
    pub fn from_scalar(tag: impl ToString, value: f32) -> Result<Summary, Error> {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(value::Value::SimpleValue(value)),
            }],
        };
        Ok(summary)
    }

    /// Build a histogram summary.
    pub fn from_histogram(
        tag: impl ToString,
        histogram: impl IntoHistogram,
    ) -> Result<Summary, Error> {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(value::Value::Histo(histogram.try_into_histogram()?)),
            }],
        };
        Ok(summary)
    }

    /// Build a tensor summary.
    pub fn from_tensor(
        tag: impl ToString,
        tensor: impl TryInto<TensorProto, Error = impl Into<Error>>,
    ) -> Result<Summary, Error> {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(value::Value::Tensor(tensor.try_into().map_err(Into::into)?)),
            }],
        };
        Ok(summary)
    }

    /// Build an image summary.
    pub fn from_image(
        tag: impl ToString,
        image: impl TryInto<Image, Error = impl Into<Error>>,
    ) -> Result<Summary, Error> {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(value::Value::Image(image.try_into().map_err(Into::into)?)),
            }],
        };
        Ok(summary)
    }

    /// Build a summary with multiple images.
    pub fn from_image_list(
        tag: impl ToString,
        images: impl IntoImageList,
    ) -> Result<Summary, Error> {
        let image_protos = images.into_image_list()?;

        let values: Vec<_> = image_protos
            .into_iter()
            .enumerate()
            .map(|(index, image_proto)| Value {
                node_name: "".into(),
                tag: format!("{}/image/{}", tag.to_string(), index),
                metadata: None,
                value: Some(value::Value::Image(image_proto)),
            })
            .collect();

        let summary = Summary { value: values };
        Ok(summary)
    }

    /// Build an audio summary.
    pub fn from_audio(
        tag: impl ToString,
        audio: impl TryInto<Audio, Error = impl Into<Error>>,
    ) -> Result<Summary, Error> {
        let summary = Summary {
            value: vec![Value {
                node_name: "".into(),
                tag: tag.to_string(),
                metadata: None,
                value: Some(value::Value::Audio(audio.try_into().map_err(Into::into)?)),
            }],
        };
        Ok(summary)
    }
}
