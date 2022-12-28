use core::ops::{Sub, SubAssign};

use crate::time::{Date, TimeStamp, WorkingDuration};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Task {
    duration: WorkingDuration,
    suggested_date: Option<Date>,
    can_be_split: bool,
    start: Option<TimeStamp>,
}

impl Task {
    #[must_use]
    pub fn new(
        duration: WorkingDuration,
        suggested_date: Option<Date>,
        can_be_split: bool,
    ) -> Self {
        Self {
            duration,
            suggested_date,
            can_be_split,
            start: None,
        }
    }

    #[must_use]
    pub fn new_with_start(
        duration: WorkingDuration,
        suggested_date: Option<Date>,
        can_be_split: bool,
        start: TimeStamp,
    ) -> Self {
        Self {
            duration,
            suggested_date,
            can_be_split,
            start: Some(start),
        }
    }

    #[must_use]
    pub fn from_duration(duration: WorkingDuration) -> Self {
        Self::new(duration, None, true)
    }

    #[must_use]
    pub fn with_duration(mut self, duration: WorkingDuration) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub fn duration(&self) -> WorkingDuration {
        self.duration
    }

    #[must_use]
    pub fn suggested_date(&self) -> Option<Date> {
        self.suggested_date
    }

    #[must_use]
    pub fn can_be_split(&self) -> bool {
        self.can_be_split
    }

    #[must_use]
    pub const fn suggested_start(&self) -> Option<TimeStamp> {
        self.start
    }
}

impl Sub<WorkingDuration> for Task {
    type Output = Self;

    fn sub(self, rhs: WorkingDuration) -> Self::Output {
        Self::new(self.duration - rhs, self.suggested_date, self.can_be_split)
    }
}

impl SubAssign<WorkingDuration> for Task {
    fn sub_assign(&mut self, rhs: WorkingDuration) {
        self.duration -= rhs;
    }
}
