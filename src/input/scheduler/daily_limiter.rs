use std::collections::HashMap;

use crate::input::scheduler::Scheduler;
use crate::time::{Date, WorkingDuration};
use crate::{min, working_duration};

/// A scheduler that schedules work exclusively on workdays.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyLimiter {
    scheduled: HashMap<Date, WorkingDuration>,
    limit: WorkingDuration,
}

impl DailyLimiter {
    const DAILY_LIMIT: WorkingDuration = working_duration!(06:00);

    #[must_use]
    pub fn new(limit: WorkingDuration) -> Self {
        Self {
            scheduled: HashMap::new(),
            limit,
        }
    }
}

impl Default for DailyLimiter {
    fn default() -> Self {
        Self::new(Self::DAILY_LIMIT)
    }
}

impl Scheduler for DailyLimiter {
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        min!(
            self.scheduled.get(&date).copied().unwrap_or_default() + wanted_duration,
            self.limit
        )
    }

    fn schedule(&mut self, date: Date, worked: WorkingDuration) {
        let scheduled = self.scheduled.entry(date).or_default();
        *scheduled += worked;
    }

    fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
        self.schedule(date, worked);
    }
}
