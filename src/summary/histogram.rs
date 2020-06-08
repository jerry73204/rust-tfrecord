use super::*;

#[derive(Debug)]
pub struct Histogram {
    pub(crate) buckets: Vec<Bucket>,
    pub(crate) min: Atomic<f64>,
    pub(crate) max: Atomic<f64>,
    pub(crate) sum: Atomic<f64>,
    pub(crate) sum_squares: Atomic<f64>,
}

impl Histogram {
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
            let mut buckets = vec![];

            if let Some(first) = limits.first() {
                if first.raw() != f64::MIN {
                    buckets.push(Bucket {
                        limit: R64::new(f64::MIN),
                        count: AtomicUsize::new(0),
                    });
                }
            }

            buckets.extend(limits.into_iter().map(|limit| Bucket {
                limit,
                count: AtomicUsize::new(0),
            }));

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

        Some(Self {
            buckets,
            min: Atomic::new(f64::MAX),
            max: Atomic::new(f64::MIN),
            sum: Atomic::new(0.0),
            sum_squares: Atomic::new(0.0),
        })
    }

    pub fn add(&self, value: R64) {
        let index = match self
            .buckets
            .binary_search_by_key(&value, |bucket| bucket.limit)
        {
            Ok(index) => index,
            Err(index) => index,
        };

        self.buckets[index].count.fetch_add(1, Ordering::SeqCst);

        // update min
        loop {
            let curr = self.min.load(Ordering::Acquire);
            let new = curr.min(value.raw());
            let swapped = self.min.compare_and_swap(curr, new, Ordering::Release);
            if swapped == curr {
                break;
            }
        }

        // update max
        loop {
            let curr = self.max.load(Ordering::Acquire);
            let new = curr.max(value.raw());
            let swapped = self.max.compare_and_swap(curr, new, Ordering::Release);
            if swapped == curr {
                break;
            }
        }

        // update sum
        loop {
            let curr = self.sum.load(Ordering::Acquire);
            let new = curr + value.raw();
            assert!(new.is_finite());
            let swapped = self.sum.compare_and_swap(curr, new, Ordering::Release);
            if swapped == curr {
                break;
            }
        }

        // update sum_square
        loop {
            let curr = self.sum_squares.load(Ordering::Acquire);
            let new = curr + value.raw().powi(2);
            assert!(new.is_finite());
            let swapped = self
                .sum_squares
                .compare_and_swap(curr, new, Ordering::Release);
            if swapped == curr {
                break;
            }
        }
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
