use std::cmp::{Ord, Ordering, PartialOrd};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input::toml_input::{self, Key};
use crate::time::{self, TimeSpan, TimeStamp};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
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

impl Entry {
    pub fn new(
        action: String,
        day: usize,
        start: TimeStamp,
        end: TimeStamp,
        pause: Option<TimeStamp>,
        vacation: Option<bool>,
    ) -> Self {
        Self {
            action,
            day,
            start,
            end,
            pause,
            vacation,
        }
    }
}

impl From<(Key, toml_input::Entry)> for Entry {
    fn from((key, entry): (Key, toml_input::Entry)) -> Self {
        Self {
            action: entry.action().to_string(),
            day: key.day(),
            start: entry.start().clone(),
            end: entry.end().clone(),
            pause: entry.pause().cloned(),
            vacation: Some(entry.is_vacation()),
        }
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }

        let mut result = self.day.cmp(&other.day);
        if result == Ordering::Equal {
            result = self.start.cmp(&other.start);
        }

        if result == Ordering::Equal {
            result = self.end.cmp(&other.end);
        }

        if result == Ordering::Equal {
            result = self.pause.cmp(&other.pause);
        }

        if result == Ordering::Equal {
            result = self.action.cmp(&other.action);
        }

        if result == Ordering::Equal {
            result = self.pause.cmp(&other.pause);
        }

        result
    }
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

    /// This returns the duration the person has worked, pauses are subtracted from the duration.
    pub fn work_duration(&self) -> Duration {
        let mut duration = self.end.elapsed(&self.start);

        if let Some(pause) = &self.pause {
            duration = duration.saturating_sub((*pause).into());
            // TODO: vacation
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

#[cfg(test)]
mod tests {
    use crate::time::DurationExt;

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_working_duration() {
        let entry = Entry {
            action: "test".to_string(),
            day: 1,
            start: TimeStamp::new(08, 00).unwrap(),
            end: TimeStamp::new(12, 23).unwrap(),
            pause: None,    // Option<TimeStamp>
            vacation: None, // Option<bool>
        };

        assert_eq!(
            entry.work_duration(),
            Duration::from_hours(4) + Duration::from_mins(23)
        );
    }
}
