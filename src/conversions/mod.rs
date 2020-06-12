use crate::{
    error::Error,
    markers::{HistogramProtoElement, TensorProtoElement, TryInfoImageList},
    protos::{
        feature::Kind, summary::Image, tensor_shape_proto::Dim, BytesList, DataType,
        Example as RawExample, Feature as RawFeature, Features, FloatList, HistogramProto,
        Int64List, TensorProto, TensorShapeProto,
    },
    types::{Example, Feature, Histogram},
};
use integer_encoding::VarInt;
use noisy_float::types::R64;
use std::{
    collections::HashMap, convert::TryFrom, io::Cursor, ops::Deref, slice, sync::atomic::Ordering,
};

mod basic_conv;
mod image_conv;
mod ndarray_conv;
mod tch_conv;

// auxiliary types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
enum ColorSpace {
    Luma = 1,
    LumaA = 2,
    Rgb = 3,
    Rgba = 4,
    DigitalYuv = 5,
    Bgra = 6,
}

struct TensorProtoInit<S>
where
    S: AsRef<[usize]>,
{
    pub shape: Option<S>,
}

impl<S> TensorProtoInit<S>
where
    S: AsRef<[usize]>,
{
    pub fn build_with_data<T>(self, data: &[T]) -> TensorProto
    where
        T: TensorProtoElement,
    {
        self.verify_shape(data.len());
        let dtype = T::DATA_TYPE as i32;
        let tensor_content = data
            .iter()
            .cloned()
            .flat_map(|elem| elem.to_bytes())
            .collect();

        TensorProto {
            dtype,
            tensor_shape: self.build_tensor_shape(),
            version_number: 0,
            tensor_content,
            half_val: vec![],
            float_val: vec![],
            double_val: vec![],
            int_val: vec![],
            string_val: vec![],
            scomplex_val: vec![],
            int64_val: vec![],
            bool_val: vec![],
            dcomplex_val: vec![],
            resource_handle_val: vec![],
            variant_val: vec![],
            uint32_val: vec![],
            uint64_val: vec![],
        }
    }

    pub fn build_string<T>(self, data: &[T]) -> TensorProto
    where
        T: AsRef<[u8]>,
    {
        self.verify_shape(data.len());
        let len_iter = data
            .iter()
            .flat_map(|bytes| (bytes.as_ref().len() as i32).encode_var_vec());
        let bytes_iter = data.iter().flat_map(|bytes| bytes.as_ref().iter().cloned());
        let tensor_content = len_iter.chain(bytes_iter).collect::<Vec<_>>();

        TensorProto {
            dtype: DataType::DtString as i32,
            tensor_shape: self.build_tensor_shape(),
            version_number: 0,
            tensor_content,
            half_val: vec![],
            float_val: vec![],
            double_val: vec![],
            int_val: vec![],
            string_val: vec![],
            scomplex_val: vec![],
            int64_val: vec![],
            bool_val: vec![],
            dcomplex_val: vec![],
            resource_handle_val: vec![],
            variant_val: vec![],
            uint32_val: vec![],
            uint64_val: vec![],
        }
    }

    fn build_tensor_shape(&self) -> Option<TensorShapeProto> {
        self.shape.as_ref().map(|shape| TensorShapeProto {
            dim: shape
                .as_ref()
                .iter()
                .cloned()
                .map(|sz| Dim {
                    size: sz as i64,
                    name: "".into(),
                })
                .collect::<Vec<_>>(),
            unknown_rank: false,
        })
    }

    fn verify_shape(&self, len: usize) {
        assert_eq!(
            len,
            self.shape
                .as_ref()
                .expect("please report bug")
                .as_ref()
                .iter()
                .fold(1, |prod, val| prod * val),
            "please report bug"
        );
    }
}
