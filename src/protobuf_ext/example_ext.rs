use crate::{
    protobuf::{Example as RawExample, Feature as RawFeature, Features},
    types::{Example, Feature},
};
use std::collections::HashMap;

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
