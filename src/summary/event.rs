use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct EventInit {
    pub wall_time: Option<f64>,
    pub step: i64,
}

impl EventInit {
    pub fn new(step: i64, wall_time: f64) -> Self {
        Self {
            wall_time: Some(wall_time),
            step,
        }
    }

    pub fn with_step(step: i64) -> Self {
        Self {
            wall_time: None,
            step,
        }
    }

    pub fn build_empty(self) -> Event {
        let (wall_time, step) = self.to_parts();
        Event {
            wall_time,
            step,
            what: None,
        }
    }

    pub fn build_with_summary(self, summary: Summary) -> Event {
        let (wall_time, step) = self.to_parts();
        Event {
            wall_time,
            step,
            what: Some(What::Summary(summary)),
        }
    }

    fn to_parts(self) -> (f64, i64) {
        let Self {
            wall_time: wall_time_opt,
            step,
        } = self;
        let wall_time = wall_time_opt.unwrap_or_else(|| Self::get_wall_time());
        (wall_time, step)
    }

    fn get_wall_time() -> f64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros() as f64
    }
}
