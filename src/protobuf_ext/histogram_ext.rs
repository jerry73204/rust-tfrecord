use crate::{error::Error, protobuf::HistogramProto, types::Histogram};
use noisy_float::prelude::*;
use num_traits::{NumCast, ToPrimitive};
use std::sync::atomic::Ordering::*;

pub use into_histogram::*;
mod into_histogram {
    use super::*;

    pub trait IntoHistogram {
        fn try_into_histogram(self) -> Result<HistogramProto, Error>;
    }

    impl<T> IntoHistogram for T
    where
        HistogramProto: TryFrom<T, Error = Error>,
    {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            self.try_into()
        }
    }

    impl IntoHistogram for Histogram {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            Ok(self.into())
        }
    }

    impl IntoHistogram for HistogramProto {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            Ok(self)
        }
    }

    impl<T> IntoHistogram for &[T]
    where
        T: Clone + ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            HistogramProto::try_from_iter(self.iter().cloned())
        }
    }

    impl<T, const SIZE: usize> IntoHistogram for [T; SIZE]
    where
        T: ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            HistogramProto::try_from_iter(self)
        }
    }

    impl<T> IntoHistogram for Vec<T>
    where
        T: ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            HistogramProto::try_from_iter(self)
        }
    }

    impl<T> IntoHistogram for &Vec<T>
    where
        T: ToPrimitive + Clone,
    {
        fn try_into_histogram(self) -> Result<HistogramProto, Error> {
            HistogramProto::try_from_iter(self.iter().cloned())
        }
    }
}

impl From<Histogram> for HistogramProto {
    fn from(from: Histogram) -> Self {
        let state = from.0.read().unwrap();

        let counts: Vec<_> = state
            .buckets
            .iter()
            .map(|bucket| bucket.count.load(Acquire) as f64)
            .collect();
        let limits: Vec<_> = state
            .buckets
            .iter()
            .map(|bucket| bucket.limit.raw())
            .collect();

        let min = state
            .stat
            .as_ref()
            .map(|stat| stat.min)
            .unwrap_or(f64::INFINITY);
        let max = state
            .stat
            .as_ref()
            .map(|stat| stat.max)
            .unwrap_or(f64::NEG_INFINITY);
        let sum = state.stat.as_ref().map(|stat| stat.sum).unwrap_or(0.0);
        let sum_squares = state
            .stat
            .as_ref()
            .map(|stat| stat.sum_squares)
            .unwrap_or(0.0);
        let len = state.stat.as_ref().map(|stat| stat.len).unwrap_or(0);

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

impl HistogramProto {
    pub fn from_iter<T, I>(iter: I) -> Self
    where
        T: ToPrimitive,
        I: IntoIterator<Item = T>,
    {
        Self::try_from_iter(iter).unwrap()
    }

    pub fn try_from_iter<T, I>(iter: I) -> Result<Self, Error>
    where
        T: ToPrimitive,
        I: IntoIterator<Item = T>,
    {
        let histogram = Histogram::default();
        iter.into_iter()
            .try_for_each(|value| -> Result<(), Error> {
                let value =
                    <R64 as NumCast>::from(value).ok_or_else(|| Error::ConversionError {
                        desc: "non-finite value found".into(),
                    })?;
                histogram.add(value);
                Ok(())
            })?;
        Ok(histogram.into())
    }
}

#[cfg(feature = "with-image")]
mod with_image {
    use super::*;
    use crate::error::Error;
    use image::{DynamicImage, ImageBuffer, Pixel};
    use std::ops::Deref;

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
        P::Subpixel: ToPrimitive,
    {
        type Error = Error;

        fn try_from(from: &ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            let components = from
                .pixels()
                .flat_map(|pixel| pixel.channels().iter().cloned());

            Self::try_from_iter(components)
        }
    }

    impl<P, C> TryFrom<ImageBuffer<P, C>> for HistogramProto
    where
        P: 'static + Pixel,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
        P::Subpixel: ToPrimitive,
    {
        type Error = Error;

        fn try_from(from: ImageBuffer<P, C>) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }

    // into histogram

    // impl<P, C> IntoHistogram for &ImageBuffer<P, C>
    // where
    //     P: 'static + Pixel,
    //     P::Subpixel: 'static,
    //     C: Deref<Target = [P::Subpixel]>,
    //     P::Subpixel: ToPrimitive,
    // {
    //     fn try_into_histogram(self) -> Result<HistogramProto, Error> {
    //         self.try_into()
    //     }
    // }

    // impl<P, C> IntoHistogram for ImageBuffer<P, C>
    // where
    //     P: 'static + Pixel,
    //     P::Subpixel: 'static,
    //     C: Deref<Target = [P::Subpixel]>,
    //     P::Subpixel: ToPrimitive,
    // {
    //     fn try_into_histogram(self) -> Result<HistogramProto, Error> {
    //         self.try_into()
    //     }
    // }

    // impl IntoHistogram for DynamicImage {
    //     fn try_into_histogram(self) -> Result<HistogramProto, Error> {
    //         self.try_into()
    //     }
    // }

    // impl IntoHistogram for &DynamicImage {
    //     fn try_into_histogram(self) -> Result<HistogramProto, Error> {
    //         self.try_into()
    //     }
    // }
}

#[cfg(feature = "with-ndarray")]
mod with_ndarray {
    use super::*;
    use ndarray::{ArrayBase, Data, Dimension, RawData};

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
}

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

    macro_rules! tensor_to_r64_vec {
        ($tensor:ident, $ty:ident) => {{
            tensor_to_vec!($tensor, $ty)
                .into_iter()
                .map(|value| {
                    let value =
                        <R64 as NumCast>::from(value).ok_or_else(|| Error::ConversionError {
                            desc: "non-finite floating point value found".into(),
                        })?;
                    Ok(value)
                })
                .collect::<Result<Vec<_>, Error>>()
        }};
    }

    impl TryFrom<&Tensor> for HistogramProto {
        type Error = Error;

        fn try_from(from: &Tensor) -> Result<Self, Self::Error> {
            let kind = from.f_kind()?;
            let values = match kind {
                Kind::Uint8 => tensor_to_r64_vec!(from, u8)?,
                Kind::Int8 => tensor_to_r64_vec!(from, i8)?,
                Kind::Int16 => tensor_to_r64_vec!(from, i16)?,
                Kind::Int => tensor_to_r64_vec!(from, i32)?,
                Kind::Int64 => tensor_to_r64_vec!(from, i64)?,
                // Kind::Half => tensor_to_r64_vec!(from, f16)?,
                Kind::Float => tensor_to_r64_vec!(from, f32)?,
                Kind::Double => tensor_to_r64_vec!(from, f64)?,
                _ => {
                    return Err(Error::ConversionError {
                        desc: format!("unsupported tensor kind {:?}", kind),
                    })
                }
            };

            Self::try_from_iter(values)
        }
    }

    impl TryFrom<Tensor> for HistogramProto {
        type Error = Error;

        fn try_from(from: Tensor) -> Result<Self, Self::Error> {
            Self::try_from(&from)
        }
    }
}
