use crate::{error::Error, protobuf::HistogramProto};
use noisy_float::prelude::*;
use num_traits::{NumCast, ToPrimitive};
use std::sync::atomic::Ordering::*;

pub use concurrent_histogram::*;
mod concurrent_histogram {
    use super::*;
    use noisy_float::types::R64;
    use std::{
        iter,
        ops::Neg,
        sync::{atomic::AtomicUsize, RwLock},
    };

    /// Concurrent histogram data structure.
    ///
    /// The methods of the histogram can be called concurrently.
    #[derive(Debug)]
    pub struct Histogram(pub(crate) RwLock<State>);

    #[derive(Debug)]
    pub(crate) struct State {
        pub(crate) buckets: Vec<Bucket>,
        pub(crate) stat: Option<Stat>,
    }

    #[derive(Debug)]
    pub(crate) struct Stat {
        pub(crate) len: usize,
        pub(crate) min: f64,
        pub(crate) max: f64,
        pub(crate) sum: f64,
        pub(crate) sum_squares: f64,
    }

    impl Histogram {
        /// Build a histogram with monotonic value limits.
        ///
        /// The values of `limits` must be monotonically increasing.
        /// Otherwise the method returns `None`.
        pub fn new(limits: Vec<R64>) -> Option<Self> {
            // check if the limit values are ordered
            let (is_ordered, _) = limits.iter().cloned().fold(
                (true, None),
                |(is_ordered, prev_limit_opt), curr_limit| {
                    let is_ordered = is_ordered
                        && prev_limit_opt
                            .map(|prev_limit| prev_limit < curr_limit)
                            .unwrap_or(true);
                    (is_ordered, Some(curr_limit))
                },
            );

            if !is_ordered {
                return None;
            }

            let buckets = {
                let mut buckets = limits
                    .into_iter()
                    .map(|limit| Bucket {
                        limit,
                        count: AtomicUsize::new(0),
                    })
                    .collect::<Vec<_>>();

                // make sure the last bucket has maximum limit
                if let Some(last) = buckets.last() {
                    if last.limit.raw() != f64::MAX {
                        buckets.push(Bucket {
                            limit: R64::new(f64::MAX),
                            count: AtomicUsize::new(0),
                        });
                    }
                }

                buckets
            };

            Some(Self(RwLock::new(State {
                buckets,
                stat: None,
            })))
        }

        /// Get the observed minimum value.
        pub fn min(&self) -> Option<f64> {
            self.0.read().unwrap().stat.as_ref()?.min.into()
        }

        /// Get the observed maximum value.
        pub fn max(&self) -> Option<f64> {
            self.0.read().unwrap().stat.as_ref()?.max.into()
        }

        /// Get the summation of contained values.
        pub fn sum(&self) -> f64 {
            self.0
                .read()
                .unwrap()
                .stat
                .as_ref()
                .map(|stat| stat.sum)
                .unwrap_or(0.0)
        }

        /// Get the summation of squares of contained values.
        pub fn sum_squares(&self) -> f64 {
            self.0
                .read()
                .unwrap()
                .stat
                .as_ref()
                .map(|stat| stat.sum_squares)
                .unwrap_or(0.0)
        }

        /// Check if there is contained values.
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// Get the number of contained values.
        pub fn len(&self) -> usize {
            self.0
                .read()
                .unwrap()
                .stat
                .as_ref()
                .map(|stat| stat.len)
                .unwrap_or(0)
        }

        /// Append a new value.
        pub fn add(&self, value: R64) {
            let mut state = self.0.write().unwrap();
            let index = match state
                .buckets
                .binary_search_by_key(&value, |bucket| bucket.limit)
            {
                Ok(index) => index,
                Err(index) => index,
            };

            state.buckets[index].count.fetch_add(1, SeqCst);

            let value = value.raw();

            let new_stat = state
                .stat
                .as_ref()
                .map(|stat| Stat {
                    len: stat.len + 1,
                    min: stat.min.min(value),
                    max: stat.max.max(value),
                    sum: stat.sum + value,
                    sum_squares: stat.sum_squares + value.powi(2),
                })
                .unwrap_or_else(|| Stat {
                    len: 1,
                    min: value,
                    max: value,
                    sum: value,
                    sum_squares: value.powi(2),
                });
            state.stat = Some(new_stat);
        }
    }

    impl Default for Histogram {
        fn default() -> Self {
            let pos_limits_iter = iter::successors(Some(R64::new(1e-12)), |prev| {
                let curr = *prev * R64::new(1.1);
                if curr.raw() < 1e20 {
                    Some(curr)
                } else {
                    None
                }
            });

            // collect negative limits
            let neg_limits = {
                let mut neg_limits = vec![R64::new(f64::MIN)];
                let mut tmp_limits = pos_limits_iter.clone().map(Neg::neg).collect::<Vec<_>>();
                tmp_limits.reverse();
                neg_limits.extend(tmp_limits);
                neg_limits
            };

            // add zero
            let mut limits = neg_limits;
            limits.push(R64::new(0.0));

            // collect positive limits
            limits.extend(pos_limits_iter);
            limits.push(R64::new(f64::MAX));

            Self::new(limits).unwrap()
        }
    }

    #[derive(Debug)]
    pub(crate) struct Bucket {
        pub(crate) limit: R64,
        pub(crate) count: AtomicUsize,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::error::Error;
        use approx::assert_abs_diff_eq;

        #[test]
        fn simple_histogram() -> Result<(), Error> {
            let histogram =
                Histogram::new(vec![R64::new(-10.0), R64::new(0.0), R64::new(10.0)]).unwrap();

            assert_eq!(histogram.len(), 0);
            assert_eq!(histogram.min(), None);
            assert_eq!(histogram.max(), None);
            assert_eq!(histogram.sum(), 0.0);
            assert_eq!(histogram.sum_squares(), 0.0);

            let values = vec![-11.0, -8.0, -6.0, 15.0, 7.0, 2.0];
            let expect_len = values.len();

            let (expect_min, expect_max, expect_sum, expect_sum_squares) = values.into_iter().fold(
                (f64::INFINITY, f64::NEG_INFINITY, 0.0, 0.0),
                |(min, max, sum, sum_squares), value| {
                    let min = min.min(value);
                    let max = max.max(value);
                    let sum = sum + value;
                    let sum_squares = sum_squares + value.powi(2);
                    histogram.add(R64::new(value));
                    (min, max, sum, sum_squares)
                },
            );

            assert_eq!(histogram.len(), expect_len);
            assert_abs_diff_eq!(histogram.max().unwrap(), expect_max);
            assert_abs_diff_eq!(histogram.min().unwrap(), expect_min);
            assert_abs_diff_eq!(histogram.sum(), expect_sum);
            assert_abs_diff_eq!(histogram.sum_squares(), expect_sum_squares);

            Ok(())
        }
    }
}

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
    #[allow(clippy::should_implement_trait)]
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
                let value = <R64 as NumCast>::from(value)
                    .ok_or_else(|| Error::conversion("non-finite value found"))?;
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
                R64::try_new(value)
                    .ok_or_else(|| Error::conversion("non-finite floating value found"))
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
                    let value = <R64 as NumCast>::from(value).ok_or_else(|| {
                        Error::conversion("non-finite floating point value found")
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
                    return Err(Error::conversion(format!(
                        "unsupported tensor kind {:?}",
                        kind
                    )))
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
