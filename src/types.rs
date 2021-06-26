//! High level example, feature and many other types.

use noisy_float::types::R64;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::{
    collections::HashMap,
    iter,
    ops::Neg,
    sync::atomic::{AtomicUsize, Ordering},
};

/// The high level example, which is a collections of named features.
pub type Example = HashMap<String, Feature>;

/// The high level feature type that stores actual data.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Feature {
    BytesList(Vec<Vec<u8>>),
    FloatList(Vec<f32>),
    Int64List(Vec<i64>),
    None,
}

mod histogram {
    use super::*;

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

            state.buckets[index].count.fetch_add(1, Ordering::SeqCst);

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
}

pub use histogram::*;

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
