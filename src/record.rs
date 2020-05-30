use crate::protos::{feature::Kind, BytesList, Example, Feature, Features, FloatList, Int64List};
use std::collections::HashMap;

pub type EasyExample = HashMap<String, EasyFeature>;

#[derive(Debug, Clone, PartialEq)]
pub enum EasyFeature {
    BytesList(Vec<Vec<u8>>),
    FloatList(Vec<f32>),
    Int64List(Vec<i64>),
    None,
}

impl From<Feature> for EasyFeature {
    fn from(from: Feature) -> Self {
        match from.kind {
            Some(Kind::BytesList(BytesList { value })) => EasyFeature::BytesList(value),
            Some(Kind::FloatList(FloatList { value })) => EasyFeature::FloatList(value),
            Some(Kind::Int64List(Int64List { value })) => EasyFeature::Int64List(value),
            None => EasyFeature::None,
        }
    }
}

impl From<EasyFeature> for Feature {
    fn from(from: EasyFeature) -> Self {
        let kind = match from {
            EasyFeature::BytesList(value) => Some(Kind::BytesList(BytesList { value })),
            EasyFeature::FloatList(value) => Some(Kind::FloatList(FloatList { value })),
            EasyFeature::Int64List(value) => Some(Kind::Int64List(Int64List { value })),
            EasyFeature::None => None,
        };
        Self { kind }
    }
}

impl From<Example> for EasyExample {
    fn from(from: Example) -> Self {
        let features = match from.features {
            Some(features) => features,
            None => return HashMap::new(),
        };
        features
            .feature
            .into_iter()
            .map(|(name, feature)| (name, EasyFeature::from(feature)))
            .collect::<HashMap<_, _>>()
    }
}

impl From<EasyExample> for Example {
    fn from(from: EasyExample) -> Self {
        let feature = from
            .into_iter()
            .map(|(name, easy_feature)| (name, Feature::from(easy_feature)))
            .collect::<HashMap<_, _>>();
        if feature.is_empty() {
            Example { features: None }
        } else {
            Example {
                features: Some(Features { feature }),
            }
        }
    }
}
