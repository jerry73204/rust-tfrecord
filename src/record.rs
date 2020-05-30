use crate::protos::{
    feature::Kind, BytesList, Feature as RawFeature, Features, FloatList, Int64List,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::protos::Example as RawExample;

pub type Example = HashMap<String, Feature>;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Feature {
    BytesList(Vec<Vec<u8>>),
    FloatList(Vec<f32>),
    Int64List(Vec<i64>),
    None,
}

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
