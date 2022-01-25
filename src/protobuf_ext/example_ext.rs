use crate::protobuf::{Example, Feature, Features};
use std::collections::HashMap;

impl Example {
    pub fn into_vec(self) -> Vec<(String, Feature)> {
        self.into_iter().collect()
    }

    pub fn into_hash_map(self) -> HashMap<String, Feature> {
        self.into_iter().collect()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, Feature)> {
        self.features
            .into_iter()
            .flat_map(|features| features.feature)
    }

    pub fn empty() -> Self {
        Self { features: None }
    }
}

impl FromIterator<(String, Feature)> for Example {
    fn from_iter<T: IntoIterator<Item = (String, Feature)>>(iter: T) -> Self {
        Self {
            features: Some(Features {
                feature: iter.into_iter().collect(),
            }),
        }
    }
}
