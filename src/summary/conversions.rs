use super::*;

// macros

macro_rules! to_f64 {
    ($value:expr, u8) => {
        $value as f64
    };
    ($value:expr, u16) => {
        $value as f64
    };
    ($value:expr, u32) => {
        $value as f64
    };
    ($value:expr, u64) => {
        $value as f64
    };
    ($value:expr, i8) => {
        $value as f64
    };
    ($value:expr, i16) => {
        $value as f64
    };
    ($value:expr, i32) => {
        $value as f64
    };
    ($value:expr, i64) => {
        $value as f64
    };
    ($value:expr, f32) => {
        $value as f64
    };
    ($value:expr, f64) => {
        $value as f64
    };
}

// traits

pub trait ToLeBytes
where
    Self: Copy,
{
    const DATA_TYPE: DataType;

    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_to_le_bytes {
    ($ty:ty, $dtype:expr) => {
        impl ToLeBytes for $ty {
            const DATA_TYPE: DataType = $dtype;

            fn to_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().iter().cloned().collect()
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

pub trait ToF64
where
    Self: Copy,
{
    fn to_f64(&self) -> f64;
}

impl ToF64 for u8 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u8)
    }
}

impl ToF64 for u16 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u16)
    }
}

impl ToF64 for u32 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u32)
    }
}

impl ToF64 for u64 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, u64)
    }
}

impl ToF64 for i8 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i8)
    }
}

impl ToF64 for i16 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i16)
    }
}

impl ToF64 for i32 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i32)
    }
}

impl ToF64 for i64 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, i64)
    }
}

impl ToF64 for f32 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, f32)
    }
}

impl ToF64 for f64 {
    fn to_f64(&self) -> f64 {
        to_f64!(*self, f64)
    }
}

// built-in Histogram to HistogramProto

impl From<&Histogram> for HistogramProto {
    fn from(from: &Histogram) -> Self {
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

impl From<Histogram> for HistogramProto {
    fn from(from: Histogram) -> Self {
        Self::from(&from)
    }
}

// slice or vec to histogram

impl From<&[f64]> for HistogramProto {
    fn from(from: &[f64]) -> Self {
        let histogram = Histogram::default();
        from.iter()
            .cloned()
            .for_each(|value| histogram.add(R64::new(value)));
        histogram.into()
    }
}

impl From<&Vec<f64>> for HistogramProto {
    fn from(from: &Vec<f64>) -> Self {
        Self::from(from.as_slice())
    }
}

impl From<Vec<f64>> for HistogramProto {
    fn from(from: Vec<f64>) -> Self {
        Self::from(from.as_slice())
    }
}

#[cfg(feature = "ndarray")]
mod ndarray_conv {
    use super::*;
    use ndarray::{ArrayBase, Data, Dimension, RawData};

    fn create_tensor_proto(
        dtype: DataType,
        shape: &[usize],
        tensor_content: Vec<u8>,
    ) -> TensorProto {
        let shape = TensorShapeProto {
            dim: shape
                .iter()
                .cloned()
                .map(|sz| Dim {
                    size: sz as i64,
                    name: "".into(),
                })
                .collect::<Vec<_>>(),
            unknown_rank: false,
        };

        TensorProto {
            dtype: dtype as i32,
            tensor_shape: Some(shape),
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
        T: ToLeBytes,
    {
        fn from(from: &ArrayBase<S, D>) -> Self {
            let content = from
                .iter()
                .flat_map(|value| value.to_bytes())
                .collect::<Vec<_>>();
            create_tensor_proto(T::DATA_TYPE, from.shape(), content)
        }
    }

    impl<S, D, T> From<ArrayBase<S, D>> for TensorProto
    where
        D: Dimension,
        S: RawData<Elem = T> + Data,
        T: ToLeBytes,
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
                    let f64_value = to_f64!(value, $ty);
                    let r64_value =
                        R64::try_new(f64_value).ok_or_else(|| Error::ConversionError {
                            desc: "non-finite floating point value found".into(),
                        })?;
                    Ok(r64_value)
                })
                .collect::<Result<Vec<_>, Error>>()
        }};
    }

    macro_rules! tensor_to_bytes {
        ($tensor:ident, $ty:ident) => {{
            let values = tensor_to_vec!($tensor, $ty);
            let content = values
                .into_iter()
                .flat_map(|value| value.to_le_bytes().iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            content
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

    // to tensor

    impl TryFrom<&Tensor> for TensorProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            let size = from.size();
            let kind = from.kind();

            let (dtype, tensor_content) = match kind {
                Kind::Uint8 => {
                    let content = tensor_to_bytes!(from, u8);
                    (DataType::DtUint8, content)
                }
                Kind::Int8 => {
                    let content = tensor_to_bytes!(from, i8);
                    (DataType::DtInt8, content)
                }
                Kind::Int16 => {
                    let content = tensor_to_bytes!(from, i16);
                    (DataType::DtInt16, content)
                }
                Kind::Int => {
                    let content = tensor_to_bytes!(from, i32);
                    (DataType::DtInt32, content)
                }
                Kind::Int64 => {
                    let content = tensor_to_bytes!(from, i64);
                    (DataType::DtInt64, content)
                }
                Kind::Float => {
                    let content = tensor_to_bytes!(from, f32);
                    (DataType::DtFloat, content)
                }
                Kind::Double => {
                    let content = tensor_to_bytes!(from, f64);
                    (DataType::DtDouble, content)
                }
                _ => {
                    return Err(Error::ConversionError {
                        desc: format!("unsupported tensor kind {:?}", kind),
                    })
                }
            };

            let shape = TensorShapeProto {
                dim: size
                    .into_iter()
                    .map(|sz| Dim {
                        size: sz,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            Ok(TensorProto {
                dtype: dtype as i32,
                tensor_shape: Some(shape),
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
            })
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
        P::Subpixel: ToF64,
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
        P::Subpixel: ToF64,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // FlatSamples to tensor

    impl<T> TryFrom<&FlatSamples<&[T]>> for TensorProto
    where
        T: ToLeBytes,
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
                .flat_map(|value| value.to_bytes())
                .collect::<Vec<_>>();
            let shape = TensorShapeProto {
                dim: vec![height as i64, width as i64, channels as i64]
                    .into_iter()
                    .map(|sz| Dim {
                        size: sz,
                        name: "".into(),
                    })
                    .collect::<Vec<_>>(),
                unknown_rank: false,
            };

            Ok(TensorProto {
                dtype: DataType::DtUint8 as i32,
                tensor_shape: Some(shape),
                version_number: 0,
                tensor_content: samples,
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
            })
        }
    }

    impl<T> TryFrom<FlatSamples<&[T]>> for TensorProto
    where
        T: ToLeBytes,
    {
        type Error = Error;

        fn try_from(from: FlatSamples<&[T]>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<T> TryFrom<&FlatSamples<&Vec<T>>> for TensorProto
    where
        T: ToLeBytes,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<&Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from.as_ref())
        }
    }

    impl<T> TryFrom<FlatSamples<&Vec<T>>> for TensorProto
    where
        T: ToLeBytes,
    {
        type Error = Error;

        fn try_from(from: FlatSamples<&Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    impl<T> TryFrom<&FlatSamples<Vec<T>>> for TensorProto
    where
        T: ToLeBytes,
    {
        type Error = Error;

        fn try_from(from: &FlatSamples<Vec<T>>) -> Result<Self, Self::Error> {
            Self::try_from(&from.as_ref())
        }
    }

    impl<T> TryFrom<FlatSamples<Vec<T>>> for TensorProto
    where
        T: ToLeBytes,
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
        T: 'static + ToLeBytes + Primitive,
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
        T: 'static + ToLeBytes + Primitive,
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
