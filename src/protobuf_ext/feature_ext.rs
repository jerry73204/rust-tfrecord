use crate::{
    protobuf::{feature::Kind, BytesList, Feature as RawFeature, FloatList, Int64List},
    types::Feature,
};

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
