use core::ops::{Sub, SubAssign};

use crate::time::{Date, TimeStamp, WorkingDuration};
use crate::utils::ArrayVec;
use crate::working_duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Task {
    duration: WorkingDuration,
    suggested_date: Option<Date>,
    can_be_split: bool,
    start: Option<TimeStamp>,
    flex: Option<usize>,
    filter: ArrayVec<Date, 31>,
}

impl Task {
    #[must_use]
    pub fn new_duration(duration: WorkingDuration) -> Self {
        Self {
            duration,
            suggested_date: None,
            can_be_split: true,
            start: None,
            flex: None,
            filter: ArrayVec::new(),
        }
    }

    pub fn new_flex(flex: usize) -> Self {
        Self {
            duration: working_duration!(00:00),
            suggested_date: None,
            can_be_split: true,
            start: None,
            flex: Some(flex),
            filter: ArrayVec::new(),
        }
    }

    #[must_use]
    pub fn flex(&self) -> Option<usize> {
        self.flex
    }

    pub fn resolve_flex(&mut self, duration: WorkingDuration) {
        self.duration = duration;
        self.flex = None;
    }

    #[must_use]
    pub fn with_start(mut self, start: TimeStamp) -> Self {
        self.start = Some(start);
        self
    }

    #[must_use]
    pub fn with_filter(mut self, dates: ArrayVec<Date, 31>) -> Self {
        self.filter = dates;
        self
    }

    #[must_use]
    pub fn with_suggested_date(mut self, date: Date) -> Self {
        self.suggested_date = Some(date);
        self
    }

    #[must_use]
    pub fn with_duration(mut self, duration: WorkingDuration) -> Self {
        self.duration = duration;
        self
    }

    pub fn has_filter(&self) -> bool {
        !self.filter.is_empty()
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

    #[must_use]
    pub fn applies_on(&self, date: Date) -> bool {
        !self.filter.contains(&date)
    }

    #[must_use]
    pub fn can_bypass_weekly_limit(&self) -> bool {
        !self.filter.is_empty()
    }
}

impl Sub<WorkingDuration> for Task {
    type Output = Self;

    fn sub(mut self, rhs: WorkingDuration) -> Self::Output {
        self.duration -= rhs;
        self
    }
}

impl SubAssign<WorkingDuration> for Task {
    fn sub_assign(&mut self, rhs: WorkingDuration) {
        *self = *self - rhs;
    }
}
