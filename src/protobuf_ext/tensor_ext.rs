use crate::{
    error::Error,
    protobuf::{tensor_shape_proto::Dim, DataType, TensorProto, TensorShapeProto},
};
use integer_encoding::VarInt;

fn numel(dims: &[Dim]) -> usize {
    dims.iter().map(|dim| dim.size as usize).product()
}

impl TensorProto {
    pub fn from_slice<T, S>(shape: S, data: &[T]) -> Result<Self, Error>
    where
        S: IntoShape,
        T: TensorProtoElement,
    {
        let dims = shape.to_shape();
        if numel(&dims) != data.len() {
            todo!();
        }

        let dtype = T::DATA_TYPE as i32;
        let tensor_content: Vec<u8> = data
            .iter()
            .flat_map(|elem| elem.to_bytes())
            .cloned()
            .collect();

        Ok(TensorProto {
            dtype,
            tensor_shape: Some(TensorShapeProto {
                dim: dims,
                unknown_rank: false,
            }),
            version_number: 0,
            tensor_content,
            ..Default::default()
        })
    }

    pub fn from_byte_slices<T, S>(shape: S, data: &[T]) -> Result<TensorProto, Error>
    where
        S: IntoShape,
        T: AsRef<[u8]>,
    {
        let dims = shape.to_shape();
        if numel(&dims) != data.len() {
            todo!();
        }
        let len_iter = data
            .iter()
            .flat_map(|bytes| (bytes.as_ref().len() as i32).encode_var_vec());
        let bytes_iter = data.iter().flat_map(|bytes| bytes.as_ref().iter().cloned());
        let tensor_content: Vec<u8> = len_iter.chain(bytes_iter).collect();

        Ok(TensorProto {
            dtype: DataType::DtString as i32,
            tensor_shape: Some(TensorShapeProto {
                dim: dims,
                unknown_rank: false,
            }),
            version_number: 0,
            tensor_content,
            ..Default::default()
        })
    }
}

pub use elem::*;
mod elem {
    use super::*;
    use bytemuck::Pod;

    /// Element types of [TensorProto](crate::protobuf::TensorProto).
    pub trait TensorProtoElement
    where
        Self: Pod,
    {
        const DATA_TYPE: DataType;

        fn to_bytes(&self) -> &[u8];
    }

    macro_rules! impl_to_le_bytes {
        ($ty:ty, $dtype:expr) => {
            impl TensorProtoElement for $ty {
                const DATA_TYPE: DataType = $dtype;

                fn to_bytes(&self) -> &[u8] {
                    bytemuck::bytes_of(self)
                }
            }
        };
    }

    impl_to_le_bytes!(u8, DataType::DtUint8);
    impl_to_le_bytes!(u16, DataType::DtUint16);
    impl_to_le_bytes!(u32, DataType::DtUint32);
    impl_to_le_bytes!(u64, DataType::DtUint64);
    impl_to_le_bytes!(i8, DataType::DtInt8);
    impl_to_le_bytes!(i16, DataType::DtInt16);
    impl_to_le_bytes!(i32, DataType::DtInt32);
    impl_to_le_bytes!(i64, DataType::DtInt64);
    impl_to_le_bytes!(f32, DataType::DtFloat);
    impl_to_le_bytes!(f64, DataType::DtDouble);
}

pub use to_shape::*;
mod to_shape {
    use super::*;
    use num::Unsigned;
    use num_traits::{NumCast, ToPrimitive};

    /// Conversion to `Vec<Dim>` type.
    ///
    /// The trait is implemented on the following types with `T` is an unsigned integer type:
    /// - slice `&[T]`
    /// - array `[T; N]`
    /// - `Vec<T>`
    pub trait IntoShape {
        fn to_shape(&self) -> Vec<Dim>;
    }

    impl<T> IntoShape for &[T]
    where
        T: Copy + Unsigned + ToPrimitive,
    {
        fn to_shape(&self) -> Vec<Dim> {
            self.iter()
                .map(|&sz| {
                    let size = <i64 as NumCast>::from(sz).expect("size is too large");
                    Dim {
                        size,
                        name: "".into(),
                    }
                })
                .collect()
        }
    }

    impl<T, const SIZE: usize> IntoShape for [T; SIZE]
    where
        T: Copy + Unsigned + ToPrimitive,
    {
        fn to_shape(&self) -> Vec<Dim> {
            self.as_slice().to_shape()
        }
    }

    impl<T> IntoShape for Vec<T>
    where
        T: Copy + Unsigned + ToPrimitive,
    {
        fn to_shape(&self) -> Vec<Dim> {
            self.iter()
                .map(|&sz| {
                    let size = <i64 as NumCast>::from(sz).expect("size is too large");
                    Dim {
                        size,
                        name: "".into(),
                    }
                })
                .collect()
        }
    }

    impl<S> IntoShape for &S
    where
        S: IntoShape,
    {
        fn to_shape(&self) -> Vec<Dim> {
            (*self).to_shape()
        }
    }
}

#[cfg(feature = "with-image")]
mod with_image {
    use super::*;
    use image::{flat::SampleLayout, DynamicImage, FlatSamples, ImageBuffer, Pixel, Primitive};
    use std::ops::Deref;

    impl<T> TryFrom<&FlatSamples<&[T]>> for TensorProto
    where
        T: TensorProtoElement,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<&[T]>) -> Result<Self, Self::Error> {
            let FlatSamples {
                layout:
                    SampleLayout {
                        width,
                        height,
                        channels,
                        ..
                    },
                ..
            } = *from;
            let samples = (0..height)
                .flat_map(|y| (0..width).flat_map(move |x| (0..channels).map(move |c| (y, x, c))))
                .map(|(y, x, c)| *from.get_sample(c, x, y).unwrap())
                .collect::<Vec<_>>();
            let shape = [height, width, channels as u32];
            let proto = TensorProto::from_slice(shape, &samples)?;

            Ok(proto)
        }
    }

    impl<T> TryFrom<FlatSamples<&[T]>> for TensorProto
    where
        T: TensorProtoElement,
    {
        type Error = Error;

        fn try_from(from: FlatSamples<&[T]>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<T> TryFrom<&FlatSamples<&Vec<T>>> for TensorProto
    where
        T: TensorProtoElement,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<&Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from.as_ref())
        }
    }

    impl<T> TryFrom<FlatSamples<&Vec<T>>> for TensorProto
    where
        T: TensorProtoElement,
    {
        type Error = Error;

        fn try_from(from: FlatSamples<&Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<T> TryFrom<&FlatSamples<Vec<T>>> for TensorProto
    where
        T: TensorProtoElement,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from.as_ref())
        }
    }

    impl<T> TryFrom<FlatSamples<Vec<T>>> for TensorProto
    where
        T: TensorProtoElement,
    {
        type Error = Error;

        fn try_from(from: FlatSamples<Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // DynamicImage to tensor

    impl TryFrom<&DynamicImage> for TensorProto {
        type Error = Error;

        fn try_from(from: &DynamicImage) -> Result<Self, Self::Error> {
            use DynamicImage::*;
            let tensor = match from {
                ImageLuma8(buffer) => Self::try_from(buffer)?,
                ImageLumaA8(buffer) => Self::try_from(buffer)?,
                ImageRgb8(buffer) => Self::try_from(buffer)?,
                ImageRgba8(buffer) => Self::try_from(buffer)?,
                ImageBgr8(buffer) => Self::try_from(buffer)?,
                ImageBgra8(buffer) => Self::try_from(buffer)?,
                ImageLuma16(buffer) => Self::try_from(buffer)?,
                ImageLumaA16(buffer) => Self::try_from(buffer)?,
                ImageRgb16(buffer) => Self::try_from(buffer)?,
                ImageRgba16(buffer) => Self::try_from(buffer)?,
            };
            Ok(tensor)
        }
    }

    impl TryFrom<DynamicImage> for TensorProto {
        type Error = Error;

        fn try_from(from: DynamicImage) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // ImageBuffer to tensor

    impl<P, C, T> TryFrom<&ImageBuffer<P, C>> for TensorProto
    where
        P: 'static + Pixel<Subpixel = T>,
        C: Deref<Target = [P::Subpixel]> + AsRef<[P::Subpixel]>,
        T: 'static + TensorProtoElement + Primitive,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }

    impl<P, C, T> TryFrom<ImageBuffer<P, C>> for TensorProto
    where
        P: 'static + Pixel<Subpixel = T>,
        C: Deref<Target = [P::Subpixel]> + AsRef<[P::Subpixel]>,
        T: 'static + TensorProtoElement + Primitive,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }
}

#[cfg(feature = "with-ndarray")]
mod with_ndarray {
    use super::*;
    use ndarray::{ArrayBase, Data, Dimension, RawData};

    impl<S, D, T> From<&ArrayBase<S, D>> for TensorProto
    where
        D: Dimension,
        S: RawData<Elem = T> + Data,
        T: TensorProtoElement,
    {
        fn from(array: &ArrayBase<S, D>) -> Self {
            let shape = array.shape();
            let array = array.as_standard_layout();
            let elems = array.as_slice().unwrap();
            TensorProto::from_slice(shape, elems).unwrap()
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
}

#[cfg(feature = "with-tch")]
pub use with_tch::*;
#[cfg(feature = "with-tch")]
mod with_tch {
    use super::*;
    use std::slice;
    use tch::{Kind, Tensor};

    macro_rules! tensor_to_vec {
        ($tensor:ident, $ty:ident) => {
            unsafe {
                let numel = $tensor.numel();
                let mut data: Vec<$ty> = Vec::with_capacity(numel);
                let slice = slice::from_raw_parts_mut(data.as_mut_ptr(), numel);
                $tensor.copy_data(slice, numel);
                data.set_len(numel);
                data
            }
        };
    }

    macro_rules! tensor_to_proto {
        ($tensor:ident, $ty:ident) => {{
            let values = tensor_to_vec!($tensor, $ty);
            let shape: Vec<_> = $tensor.size().into_iter().map(|sz| sz as usize).collect();
            TensorProto::from_slice(shape, &values)
        }};
    }

    impl TryFrom<&Tensor> for TensorProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            let kind = from.f_kind()?;
            match kind {
                Kind::Uint8 => tensor_to_proto!(from, u8),
                Kind::Int8 => tensor_to_proto!(from, i8),
                Kind::Int16 => tensor_to_proto!(from, i16),
                Kind::Int => tensor_to_proto!(from, i32),
                Kind::Int64 => tensor_to_proto!(from, i64),
                Kind::Float => tensor_to_proto!(from, f32),
                Kind::Double => tensor_to_proto!(from, f64),
                _ => {
                    return Err(Error::conversion(format!(
                        "unsupported tensor kind {:?}",
                        kind
                    )))
                }
            }
        }
    }

    impl TryFrom<Tensor> for TensorProto {
        type Error = Error;
        fn try_from(from: Tensor) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }
}
