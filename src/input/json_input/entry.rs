use std::cmp::{Ord, Ordering, PartialOrd};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input::toml_input::{self, Key};
use crate::time::{TimeSpan, TimeStamp, WorkingDuration};
use crate::working_duration;

#[must_use]
const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Entry {
    action: String,
    day: usize,
    start: TimeStamp,
    end: TimeStamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pause: Option<WorkingDuration>,
    #[serde(skip_serializing_if = "is_false", default)]
    vacation: bool,
}

impl Entry {
    pub fn new(
        action: impl ToString,
        day: usize,
        start: TimeStamp,
        end: TimeStamp,
        pause: Option<WorkingDuration>,
    ) -> Self {
        Self {
            action: action.to_string(),
            day,
            start,
            end,
            pause,
            vacation: false,
        }
    }

    pub fn new_vacation(
        action: impl ToString,
        day: usize,
        start: TimeStamp,
        end: TimeStamp,
    ) -> Self {
        Self {
            action: action.to_string(),
            day,
            start,
            end,
            pause: None,
            vacation: true,
        }
    }
}

impl From<(Key, toml_input::Entry)> for Entry {
    fn from((key, entry): (Key, toml_input::Entry)) -> Self {
        Self {
            action: entry.action().to_string(),
            day: key.day(),
            start: entry.start(),
            end: entry.end(),
            pause: entry.pause(),
            vacation: entry.is_vacation(),
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
#[error("Exceeded maximum allowed work time by {} on day {}", duration, day)]
pub struct ExceededWorkTime {
    duration: WorkingDuration,
    day: usize,
}

impl ExceededWorkTime {
    pub fn new(duration: WorkingDuration, day: usize) -> Self {
        Self { duration, day }
    }
}

impl Entry {
    /// This returns the duration that has been worked,
    /// pauses are subtracted from the duration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use time_sheet::input::json_input::Entry;
    /// # use time_sheet::{time_stamp, working_duration};
    /// #
    /// assert_eq!(
    ///     Entry::new(
    ///         "made breakfast",
    ///         1,
    ///         time_stamp!(08:00),
    ///         time_stamp!(08:45),
    ///         None,
    ///     ).work_duration(),
    ///     working_duration!(00:45),
    /// );
    ///
    /// assert_eq!(
    ///     Entry::new(
    ///         "made breakfast",
    ///         2,
    ///         time_stamp!(08:00),
    ///         time_stamp!(08:45),
    ///         Some(working_duration!(00:15)),
    ///     ).work_duration(),
    ///     working_duration!(00:30),
    /// );
    /// ```
    ///
    /// If the entry is a vacation, the duration will always be zero:
    ///
    /// ```
    /// # use time_sheet::input::json_input::Entry;
    /// # use time_sheet::{time_stamp, working_duration};
    /// #
    /// assert_eq!(
    ///     Entry::new_vacation(
    ///         "christmas vacation",
    ///         24,
    ///         time_stamp!(00:00),
    ///         time_stamp!(23:59),
    ///     ).work_duration(),
    ///     working_duration!(00:00),
    /// );
    /// ```
    pub fn work_duration(&self) -> WorkingDuration {
        if self.is_vacation() {
            return working_duration!(00:00);
        }

        self.time_span().duration() - self.break_duration()
    }

    pub fn break_duration(&self) -> WorkingDuration {
        self.pause.unwrap_or_default()
    }

    // TODO: might be better to return Transfer with a method that makes the error?
    pub fn remaining_work_time(
        &self,
        maximum: WorkingDuration,
    ) -> Result<WorkingDuration, ExceededWorkTime> {
        let work_duration = self.work_duration();
        if work_duration > maximum {
            return Err(ExceededWorkTime::new(work_duration, self.day));
        }

        Ok(maximum - self.work_duration())
    }

    pub fn day(&self) -> usize {
        self.day
    }

    pub fn time_span(&self) -> TimeSpan {
        TimeSpan::new(self.start, self.end)
    }

    pub const fn is_vacation(&self) -> bool {
        self.vacation
    }
}

#[cfg(test)]
mod tests {
    use crate::time_stamp;

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_working_duration() {
        assert_eq!(
            Entry::new(
                "did something",
                1,
                time_stamp!(08:00),
                time_stamp!(12:23),
                None
            )
            .work_duration(),
            working_duration!(04:23),
        );

        assert_eq!(
            Entry::new(
                "did something",
                1,
                time_stamp!(08:00),
                time_stamp!(14:45),
                Some(working_duration!(01:15))
            )
            .work_duration(),
            working_duration!(05:30),
        );
    }
}
