use crate::input::scheduler::Scheduler;
use crate::time::{Date, WorkingDuration};
use crate::working_duration;

/// A scheduler that schedules work exclusively on workdays.
pub struct WorkdayScheduler {}

impl WorkdayScheduler {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Scheduler for WorkdayScheduler {
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        if date.is_workday() {
            wanted_duration
        } else {
            working_duration!(00:00)
        }
    }
}
