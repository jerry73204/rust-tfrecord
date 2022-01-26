use std::{borrow::Cow, cmp, iter, ops::Neg};

use crate::{
    ensure_argument,
    error::{Error, Result},
    protobuf::HistogramProto,
};
use itertools::{chain, izip};
use noisy_float::prelude::*;
use num_traits::{NumCast, ToPrimitive};

impl HistogramProto {
    pub fn new<'a, L>(bucket_limit: L) -> Result<Self>
    where
        L: Into<Cow<'a, [f64]>>,
    {
        let bucket_limit = bucket_limit.into();

        let is_finite = bucket_limit.iter().all(|&val| val.is_finite());
        if !is_finite {
            return Err(Error::invalid_argument(
                "bucket_limit must be finite numbers",
            ));
        }

        let is_ordered =
            izip!(bucket_limit.as_ref(), &bucket_limit[1..]).all(|(&lhs, &rhs)| lhs < rhs);
        if !is_ordered {
            return Err(Error::invalid_argument(
                "bucket_limit must be monotonically ordered",
            ));
        }

        let bucket_limit = bucket_limit.into_owned();

        Ok(Self {
            min: f64::MAX,
            max: f64::MIN,
            num: 0.0,
            sum: 0.0,
            sum_squares: 0.0,
            bucket: vec![0.0; bucket_limit.len()],
            bucket_limit,
        })
    }

    pub fn tf_default() -> Self {
        let pos_limits: Vec<_> = iter::successors(Some(1e-12), |prev| {
            let curr = *prev * 1.1;
            let ok = curr < 1e20;
            ok.then(|| curr)
        })
        .collect();

        let limits: Vec<_> = chain!(
            pos_limits.iter().cloned().map(Neg::neg).rev(),
            [0.0],
            pos_limits.iter().cloned(),
        )
        .collect();

        Self::new(limits).unwrap()
    }

    pub fn add_one(&mut self, value: f64) {
        self.try_add_one(value).unwrap()
    }

    pub fn add(&mut self, value: f64, count: f64) {
        self.try_add(value, count).unwrap()
    }

    pub fn try_add_one(&mut self, value: f64) -> Result<()> {
        self.try_add(value, 1.0)
    }

    pub fn try_add(&mut self, value: f64, count: f64) -> Result<()> {
        ensure_argument!(
            value.is_finite(),
            "inserted value must be finite, but get {}",
            value
        );
        ensure_argument!(
            count.is_finite() && !count.is_sign_negative(),
            "inserted count must be finite and non-negative, but get {}",
            count
        );
        ensure_argument!(
            self.bucket_limit.len() == self.bucket.len(),
            "the lengths of bucket_limit and bucket fields must be equal"
        );

        let index = match self
            .bucket_limit
            .binary_search_by_key(&r64(value), |&limit| r64(limit))
        {
            Ok(index) => index,
            Err(index) => index,
        };

        if index < self.bucket_limit.len() {
            self.bucket[index] += count;
            self.num += 1.0;
            self.sum += value;
            self.sum_squares += value.powi(2);
            self.min = cmp::min(r64(self.min), r64(value)).raw();
            self.max = cmp::max(r64(self.max), r64(value)).raw();
        }

        Ok(())
    }

    pub fn try_iter(&self) -> Result<impl Iterator<Item = (f64, f64)> + '_> {
        ensure_argument!(
            self.bucket_limit.len() == self.bucket.len(),
            "the lengths of bucket_limit and bucket fields must be equal"
        );
        let iter = izip!(
            self.bucket_limit.iter().cloned(),
            self.bucket.iter().cloned(),
        );
        Ok(iter)
    }

    pub fn iter(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.try_iter().unwrap()
    }

    pub fn try_from_iter<T, I>(iter: I) -> Result<Self, Error>
    where
        T: ToPrimitive,
        I: IntoIterator<Item = T>,
    {
        let mut histogram = Self::tf_default();
        iter.into_iter().try_for_each(|value| -> Result<_> {
            let value = <f64 as NumCast>::from(value)
                .ok_or_else(|| Error::invalid_argument("invalid value"))?;
            histogram.try_add_one(value)?;
            Ok(())
        })?;
        Ok(histogram)
    }
}

impl<A> FromIterator<A> for HistogramProto
where
    A: ToPrimitive,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self::try_from_iter(iter).unwrap()
    }
}

pub use into_histogram::*;
mod into_histogram {
    use super::*;

    pub trait IntoHistogram {
        fn try_into_histogram(self) -> Result<HistogramProto>;
    }

    impl IntoHistogram for HistogramProto {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            Ok(self)
        }
    }

    impl<T> IntoHistogram for &[T]
    where
        T: Clone + ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            HistogramProto::try_from_iter(self.iter().cloned())
        }
    }

    impl<T, const SIZE: usize> IntoHistogram for [T; SIZE]
    where
        T: ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            HistogramProto::try_from_iter(self)
        }
    }

    impl<T> IntoHistogram for Vec<T>
    where
        T: ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            HistogramProto::try_from_iter(self)
        }
    }

    impl<T> IntoHistogram for &Vec<T>
    where
        T: ToPrimitive + Clone,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            HistogramProto::try_from_iter(self.iter().cloned())
        }
    }
}

#[cfg(feature = "with-image")]
mod with_image {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Pixel};
    use std::ops::Deref;

    // DynamicImage to histogram
    impl IntoHistogram for &DynamicImage {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            use DynamicImage::*;
            let histogram = match self {
                ImageLuma8(buffer) => buffer.try_into_histogram()?,
                ImageLumaA8(buffer) => buffer.try_into_histogram()?,
                ImageRgb8(buffer) => buffer.try_into_histogram()?,
                ImageRgba8(buffer) => buffer.try_into_histogram()?,
                ImageBgr8(buffer) => buffer.try_into_histogram()?,
                ImageBgra8(buffer) => buffer.try_into_histogram()?,
                ImageLuma16(buffer) => buffer.try_into_histogram()?,
                ImageLumaA16(buffer) => buffer.try_into_histogram()?,
                ImageRgb16(buffer) => buffer.try_into_histogram()?,
                ImageRgba16(buffer) => buffer.try_into_histogram()?,
            };
            Ok(histogram)
        }
    }

    impl IntoHistogram for DynamicImage {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            (&self).try_into_histogram()
        }
    }

    // ImageBuffer to histogram
    impl<P, C> IntoHistogram for &ImageBuffer<P, C>
    where
        P: 'static + Pixel,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
        P::Subpixel: ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            let components = self
                .pixels()
                .flat_map(|pixel| pixel.channels().iter().cloned());
            HistogramProto::try_from_iter(components)
        }
    }

    impl<P, C> IntoHistogram for ImageBuffer<P, C>
    where
        P: 'static + Pixel,
        P::Subpixel: 'static,
        C: Deref<Target = [P::Subpixel]>,
        P::Subpixel: ToPrimitive,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            (&self).try_into_histogram()
        }
    }
}

#[cfg(feature = "with-ndarray")]
mod with_ndarray {
    use super::*;
    use ndarray::{ArrayBase, Data, Dimension, RawData};

    impl<S, D> IntoHistogram for &ArrayBase<S, D>
    where
        S: RawData<Elem = f64> + Data,
        D: Dimension,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            Ok(self.iter().cloned().collect())
        }
    }

    impl<S, D> IntoHistogram for ArrayBase<S, D>
    where
        S: RawData<Elem = f64> + Data,
        D: Dimension,
    {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            (&self).try_into_histogram()
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
                    let value = <R64 as NumCast>::from(value).ok_or_else(|| {
                        Error::conversion("non-finite floating point value found")
                    })?;
                    Ok(value)
                })
                .collect::<Result<Vec<_>, Error>>()
        }};
    }

    impl IntoHistogram for &Tensor {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            let kind = self.f_kind()?;
            let values = match kind {
                Kind::Uint8 => tensor_to_r64_vec!(self, u8)?,
                Kind::Int8 => tensor_to_r64_vec!(self, i8)?,
                Kind::Int16 => tensor_to_r64_vec!(self, i16)?,
                Kind::Int => tensor_to_r64_vec!(self, i32)?,
                Kind::Int64 => tensor_to_r64_vec!(self, i64)?,
                // Kind::Half => tensor_to_r64_vec!(self, f16)?,
                Kind::Float => tensor_to_r64_vec!(self, f32)?,
                Kind::Double => tensor_to_r64_vec!(self, f64)?,
                _ => {
                    return Err(Error::conversion(format!(
                        "unsupported tensor kind {:?}",
                        kind
                    )))
                }
            };

            HistogramProto::try_from_iter(values)
        }
    }

    impl IntoHistogram for Tensor {
        fn try_into_histogram(self) -> Result<HistogramProto> {
            (&self).try_into_histogram()
        }
    }
}
