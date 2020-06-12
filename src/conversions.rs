use crate::{
    error::Error,
    markers::{HistogramProtoElement, TensorProtoElement},
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

// auxiliary types

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

// protobuf Feature from/to crate's Feature

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

// protobuf Example from/to crate's Example

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

// built-in Histogram to HistogramProto

impl From<Histogram> for HistogramProto {
    fn from(from: Histogram) -> Self {
        let Histogram {
            buckets,
            len,
            min,
            max,
            sum,
            sum_squares,
        } = from;

        let min = min.load(Ordering::Relaxed);
        let max = max.load(Ordering::Relaxed);
        let sum = sum.load(Ordering::Relaxed);
        let sum_squares = sum_squares.load(Ordering::Relaxed);
        let len = len.load(Ordering::Relaxed);

        let counts = buckets
            .iter()
            .map(|bucket| bucket.count.load(Ordering::Relaxed) as f64)
            .collect::<Vec<_>>();
        let limits = buckets
            .iter()
            .map(|bucket| bucket.limit.raw())
            .collect::<Vec<_>>();

        Self {
            min,
            max,
            num: len as f64,
            sum,
            sum_squares,
            bucket_limit: limits,
            bucket: counts,
        }
    }
}

// slice or vec to histogram

impl<T> From<&[T]> for HistogramProto
where
    T: HistogramProtoElement,
{
    fn from(from: &[T]) -> Self {
        let histogram = Histogram::default();
        from.iter()
            .cloned()
            .for_each(|value| histogram.add(R64::new(value.to_f64())));
        histogram.into()
    }
}

impl<T> From<&Vec<T>> for HistogramProto
where
    T: HistogramProtoElement,
{
    fn from(from: &Vec<T>) -> Self {
        Self::from(from.as_slice())
    }
}

impl<T> From<Vec<T>> for HistogramProto
where
    T: HistogramProtoElement,
{
    fn from(from: Vec<T>) -> Self {
        Self::from(from.as_slice())
    }
}

// slice or vec to TensorProto

impl<S> From<&[S]> for TensorProto
where
    S: AsRef<[u8]>,
{
    fn from(from: &[S]) -> Self {
        TensorProtoInit {
            shape: Some(vec![from.len()]),
        }
        .build_string(from)
    }
}

impl<S> From<&Vec<S>> for TensorProto
where
    S: AsRef<[u8]>,
{
    fn from(from: &Vec<S>) -> Self {
        From::<&[_]>::from(from.as_ref())
    }
}

impl<S> From<Vec<S>> for TensorProto
where
    S: AsRef<[u8]>,
{
    fn from(from: Vec<S>) -> Self {
        From::<&[_]>::from(from.as_ref())
    }
}

#[cfg(feature = "ndarray")]
mod ndarray_conv {
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
}

#[cfg(feature = "tch")]
mod tch_conv {
    use super::*;
    use tch::{Kind, Tensor};

    // macros

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

    macro_rules! tensor_to_r64_vec {
        ($tensor:ident, $ty:ident) => {{
            tensor_to_vec!($tensor, $ty)
                .into_iter()
                .map(|value| {
                    let f64_value = value as f64;
                    let r64_value =
                        R64::try_new(f64_value).ok_or_else(|| Error::ConversionError {
                            desc: "non-finite floating point value found".into(),
                        })?;
                    Ok(r64_value)
                })
                .collect::<Result<Vec<_>, Error>>()
        }};
    }

    macro_rules! tensor_to_proto {
        ($tensor:ident, $ty:ident) => {{
            let values = tensor_to_vec!($tensor, $ty);
            let size = $tensor
                .size()
                .into_iter()
                .map(|sz| sz as usize)
                .collect::<Vec<_>>();
            TensorProtoInit { shape: Some(size) }.build_with_data(&values)
        }};
    }

    // to histogram

    impl TryFrom<&Tensor> for HistogramProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            let kind = from.kind();
            let values = match kind {
                Kind::Uint8 => tensor_to_r64_vec!(from, u8)?,
                Kind::Int8 => tensor_to_r64_vec!(from, i8)?,
                Kind::Int16 => tensor_to_r64_vec!(from, i16)?,
                Kind::Int => tensor_to_r64_vec!(from, i32)?,
                Kind::Int64 => tensor_to_r64_vec!(from, i64)?,
                Kind::Float => tensor_to_r64_vec!(from, f32)?,
                Kind::Double => tensor_to_r64_vec!(from, f64)?,
                _ => {
                    return Err(Error::ConversionError {
                        desc: format!("unsupported tensor kind {:?}", kind),
                    })
                }
            };

            let histogram = Histogram::default();
            values.into_iter().for_each(|value| histogram.add(value));
            Ok(histogram.into())
        }
    }

    impl TryFrom<Tensor> for HistogramProto {
        type Error = Error;

        fn try_from(from: Tensor) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // to TensorProto

    impl TryFrom<&Tensor> for TensorProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            // let size = from.size();
            let kind = from.kind();
            let proto = match kind {
                Kind::Uint8 => tensor_to_proto!(from, u8),
                Kind::Int8 => tensor_to_proto!(from, i8),
                Kind::Int16 => tensor_to_proto!(from, i16),
                Kind::Int => tensor_to_proto!(from, i32),
                Kind::Int64 => tensor_to_proto!(from, i64),
                Kind::Float => tensor_to_proto!(from, f32),
                Kind::Double => tensor_to_proto!(from, f64),
                _ => {
                    return Err(Error::ConversionError {
                        desc: format!("unsupported tensor kind {:?}", kind),
                    })
                }
            };

            Ok(proto)
        }
    }

    impl TryFrom<Tensor> for TensorProto {
        type Error = Error;
        fn try_from(from: Tensor) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }
}

#[cfg(feature = "image")]
mod image_conv {
    use super::*;
    use image::{
        flat::SampleLayout, png::PNGEncoder, ColorType, DynamicImage, FlatSamples, ImageBuffer,
        Pixel, Primitive,
    };

    // auxiliary types and traits

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

    impl TryFrom<ColorType> for ColorSpace {
        type Error = Error;

        fn try_from(from: ColorType) -> Result<Self, Self::Error> {
            let color_space = match from {
                ColorType::L8 => ColorSpace::Luma,
                ColorType::La8 => ColorSpace::LumaA,
                ColorType::Rgb8 => ColorSpace::Rgb,
                ColorType::Rgba8 => ColorSpace::Rgba,
                ColorType::Bgra8 => ColorSpace::Bgra,
                ColorType::L16 => ColorSpace::Luma,
                ColorType::La16 => ColorSpace::LumaA,
                ColorType::Rgb16 => ColorSpace::Rgb,
                ColorType::Rgba16 => ColorSpace::Rgba,
                _ => {
                    return Err(Error::ConversionError {
                        desc: "color space is not supported".into(),
                    });
                }
            };
            Ok(color_space)
        }
    }

    // DynamicImage to histogram

    impl TryFrom<&DynamicImage> for HistogramProto {
        type Error = Error;

        fn try_from(from: &DynamicImage) -> Result<Self, Self::Error> {
            use DynamicImage::*;
            let histogram = match from {
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
            Ok(histogram)
        }
    }

    impl TryFrom<DynamicImage> for HistogramProto {
        type Error = Error;
        fn try_from(from: DynamicImage) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // ImageBuffer to histogram

    impl<P, C> TryFrom<&ImageBuffer<P, C>> for HistogramProto
    where
        P: 'static + Pixel,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
        P::Subpixel: HistogramProtoElement,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            let components_iter = from
                .pixels()
                .flat_map(|pixel| pixel.channels().iter().cloned().collect::<Vec<_>>())
                .map(|component| {
                    R64::try_new(component.to_f64()).ok_or_else(|| Error::ConversionError {
                        desc: "non-finite value found".into(),
                    })
                });

            let histogram = Histogram::default();
            for result in components_iter {
                let component = result?;
                histogram.add(component);
            }

            Ok(histogram.into())
        }
    }

    impl<P, C> TryFrom<ImageBuffer<P, C>> for HistogramProto
    where
        P: 'static + Pixel,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
        P::Subpixel: HistogramProtoElement,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // FlatSamples to tensor

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
            let shape = vec![height as usize, width as usize, channels as usize];
            let proto = TensorProtoInit { shape: Some(shape) }.build_with_data(&samples);

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

    // DynamicImage to image

    impl TryFrom<&DynamicImage> for Image {
        type Error = Error;

        fn try_from(from: &DynamicImage) -> Result<Self, Self::Error> {
            use DynamicImage::*;
            let image = match from {
                ImageLuma8(buffer) => Self::try_from(buffer)?,
                ImageLumaA8(buffer) => Self::try_from(buffer)?,
                ImageRgb8(buffer) => Self::try_from(buffer)?,
                ImageRgba8(buffer) => Self::try_from(buffer)?,
                ImageBgra8(buffer) => Self::try_from(buffer)?,
                _ => {
                    return Err(Error::ConversionError {
                        desc: format!("unsupported image type"),
                    });
                }
            };
            Ok(image)
        }
    }

    impl TryFrom<DynamicImage> for Image {
        type Error = Error;

        fn try_from(from: DynamicImage) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // FlatSamples to image

    impl<B> TryFrom<&FlatSamples<B>> for Image
    where
        B: AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<B>) -> Result<Self, Self::Error> {
            let FlatSamples {
                layout:
                    SampleLayout {
                        width,
                        height,
                        channels,
                        ..
                    },
                color_hint,
                ..
            } = *from;
            let color_type = color_hint.ok_or_else(|| Error::ConversionError {
                desc: "color_hint must not be None".into(),
            })?;
            let colorspace = ColorSpace::try_from(color_type)?;
            let samples = (0..height)
                .flat_map(|y| (0..width).flat_map(move |x| (0..channels).map(move |c| (y, x, c))))
                .map(|(y, x, c)| *from.get_sample(c, x, y).unwrap())
                .collect::<Vec<_>>();

            let encoded_image_string = {
                let mut cursor = Cursor::new(vec![]);
                PNGEncoder::new(&mut cursor)
                    .encode(&samples, width, height, color_type)
                    .map_err(|err| Error::ConversionError {
                        desc: format!("{:?}", err),
                    })?;
                cursor.into_inner()
            };

            Ok(Image {
                height: height as i32,
                width: width as i32,
                colorspace: colorspace as i32,
                encoded_image_string,
            })
        }
    }

    impl<B> TryFrom<FlatSamples<B>> for Image
    where
        B: AsRef<[u8]>,
    {
        type Error = Error;

        fn try_from(from: FlatSamples<B>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // ImageBuffer to image

    impl<P, C> TryFrom<&ImageBuffer<P, C>> for Image
    where
        P: 'static + Pixel<Subpixel = u8>,
        C: Deref<Target = [P::Subpixel]> + AsRef<[P::Subpixel]>,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(from.as_flat_samples())
        }
    }
}
