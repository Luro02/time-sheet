use std::collections::HashMap;

use crate::input::scheduler::{Scheduler, SchedulerOptions};
use crate::min;
use crate::time::{Date, WorkingDuration};

/// A scheduler that limits the amount of work per day.
#[derive(Debug, Clone, PartialEq)]
pub struct DailyLimiter {
    scheduled: HashMap<Date, WorkingDuration>,
    limit: WorkingDuration,
}

impl DailyLimiter {
    #[must_use]
    pub fn new(options: &SchedulerOptions) -> Self {
        Self {
            scheduled: HashMap::new(),
            limit: options.daily_limit,
        }
    }

    #[must_use]
    pub const fn limit(&self) -> WorkingDuration {
        self.limit
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
