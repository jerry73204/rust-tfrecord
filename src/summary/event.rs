use crate::protobuf::{event::What, Event, Summary};
use std::time::SystemTime;

/// A [Event] initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct EventMeta {
    /// The wall clock time in microseconds.
    ///
    /// If the field is set to `None`, it sets to current system time when the event is built.
    pub wall_time: Option<f64>,
    /// The global step.
    pub step: i64,
}

impl EventMeta {
    /// Create a initializer with global step and wall time.
    pub fn new(step: i64, wall_time: f64) -> Self {
        Self {
            wall_time: Some(wall_time),
            step,
        }
    }

    /// Create a initializer with global step and without wall time.
    pub fn with_step(step: i64) -> Self {
        Self {
            wall_time: None,
            step,
        }
    }

    /// Build an empty event.
    pub fn build_empty(&self) -> Event {
        let (wall_time, step) = self.to_parts();
        Event {
            wall_time,
            step,
            what: None,
        }
    }

    /// Build an event with a summary.
    pub fn build_with_summary(&self, summary: Summary) -> Event {
        let (wall_time, step) = self.to_parts();
        Event {
            wall_time,
            step,
            what: Some(What::Summary(summary)),
        }
    }

    fn to_parts(&self) -> (f64, i64) {
        let Self {
            wall_time: wall_time_opt,
            step,
        } = *self;
        let wall_time = wall_time_opt.unwrap_or_else(wall_time_now);
        (wall_time, step)
    }
}

impl From<i64> for EventMeta {
    fn from(step: i64) -> Self {
        Self::with_step(step)
    }
}

impl From<(i64, f64)> for EventMeta {
    fn from((step, wall_time): (i64, f64)) -> Self {
        Self::new(step, wall_time)
    }
}

impl From<(i64, SystemTime)> for EventMeta {
    fn from((step, time): (i64, SystemTime)) -> Self {
        Self::new(
            step,
            time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as f64
                / 1.0e9,
        )
    }
}

fn wall_time_now() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f64
        / 1.0e9
}
