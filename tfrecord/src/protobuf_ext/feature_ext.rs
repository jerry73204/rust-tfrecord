use crate::protobuf::{feature::Kind, BytesList, Feature, FloatList, Int64List};
use std::borrow::Cow;

/// Enumeration of feature kinds returned from [Feature::into_kinds()]
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureKind {
    Bytes(Vec<Vec<u8>>),
    F32(Vec<f32>),
    I64(Vec<i64>),
}

impl Feature {
    pub fn into_kinds(self) -> Option<FeatureKind> {
        match self.kind {
            Some(Kind::BytesList(BytesList { value })) => Some(FeatureKind::Bytes(value)),
            Some(Kind::FloatList(FloatList { value })) => Some(FeatureKind::F32(value)),
            Some(Kind::Int64List(Int64List { value })) => Some(FeatureKind::I64(value)),
            None => None,
        }
    }

    pub fn from_bytes_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Vec<u8>>,
    {
        Self {
            kind: Some(Kind::BytesList(BytesList {
                value: iter.into_iter().collect(),
            })),
        }
    }

    pub fn from_f32_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = f32>,
    {
        Self {
            kind: Some(Kind::FloatList(FloatList {
                value: iter.into_iter().collect(),
            })),
        }
    }

    pub fn from_i64_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = i64>,
    {
        Self {
            kind: Some(Kind::Int64List(Int64List {
                value: iter.into_iter().collect(),
            })),
        }
    }

    pub fn from_bytes_list<'a, F>(list: F) -> Self
    where
        F: Into<Cow<'a, [Vec<u8>]>>,
    {
        Self {
            kind: Some(Kind::BytesList(BytesList {
                value: list.into().into_owned(),
            })),
        }
    }

    pub fn from_f32_list<'a, F>(list: F) -> Self
    where
        F: Into<Cow<'a, [f32]>>,
    {
        Self {
            kind: Some(Kind::FloatList(FloatList {
                value: list.into().into_owned(),
            })),
        }
    }

    pub fn from_i64_list<'a, F>(list: F) -> Self
    where
        F: Into<Cow<'a, [i64]>>,
    {
        Self {
            kind: Some(Kind::Int64List(Int64List {
                value: list.into().into_owned(),
            })),
        }
    }

    pub fn empty() -> Self {
        Self { kind: None }
    }

    pub fn as_bytes_list(&self) -> Option<&[Vec<u8>]> {
        if let Some(Kind::BytesList(BytesList { value })) = &self.kind {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_f32_list(&self) -> Option<&[f32]> {
        if let Some(Kind::FloatList(FloatList { value })) = &self.kind {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_i64_list(&self) -> Option<&[i64]> {
        if let Some(Kind::Int64List(Int64List { value })) = &self.kind {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_bytes_list(self) -> Result<Vec<Vec<u8>>, Self> {
        if let Some(Kind::BytesList(BytesList { value })) = self.kind {
            Ok(value)
        } else {
            Err(self)
        }
    }

    pub fn into_f32_list(self) -> Result<Vec<f32>, Self> {
        if let Some(Kind::FloatList(FloatList { value })) = self.kind {
            Ok(value)
        } else {
            Err(self)
        }
    }

    pub fn into_i64_list(self) -> Result<Vec<i64>, Self> {
        if let Some(Kind::Int64List(Int64List { value })) = self.kind {
            Ok(value)
        } else {
            Err(self)
        }
    }
}
