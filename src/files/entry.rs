use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::time::{self, TimeSpan, TimeStamp};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Entry {
    action: String,
    day: usize,
    start: TimeStamp,
    end: TimeStamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pause: Option<TimeStamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vacation: Option<bool>,
}

#[derive(Debug, Clone, Error)]
#[error(
    "Exceeded maximum allowed work time by {} on day {}",
    time::format_duration(duration),
    day
)]
pub struct ExceededWorkTime {
    duration: Duration,
    day: usize,
}

impl ExceededWorkTime {
    pub fn new(duration: Duration, day: usize) -> Self {
        Self { duration, day }
    }
}

impl Entry {
    const MAX_WORK_TIME_DAY: Duration = time::duration_from_hours(10);

    fn total_duration(&self) -> Duration {
        self.end.elapsed(&self.start)
    }

    /// This returns the duration the person has worked, pauses are subtracted from the duration.
    pub fn work_duration(&self) -> Duration {
        let mut duration = self.end.elapsed(&self.start);

        if let Some(pause) = &self.pause {
            duration = duration.saturating_sub((*pause).into());
        }

        duration
    }

    pub fn break_duration(&self) -> Duration {
        self.pause.map(Into::into).unwrap_or_default()
    }

    pub fn remaining_work_time(&self) -> Result<Duration, ExceededWorkTime> {
        let work_duration = self.work_duration();
        if work_duration > Self::MAX_WORK_TIME_DAY {
            return Err(ExceededWorkTime::new(work_duration, self.day));
        }

        Ok(Self::MAX_WORK_TIME_DAY - self.work_duration())
    }

    pub fn day(&self) -> usize {
        self.day
    }

    pub fn start(&self) -> TimeStamp {
        self.start
    }

    pub fn end(&self) -> TimeStamp {
        self.end
    }

    pub fn time_span(&self) -> TimeSpan {
        TimeSpan::new(self.start, self.end)
    }
}
