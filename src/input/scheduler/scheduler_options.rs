use crate::time::WorkingDuration;
use crate::working_duration;

/// Options to configure the default scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SchedulerOptions {
    /// If this is set to `true`, tasks can be scheduled on days where the user
    /// has fixed entries.
    pub should_schedule_with_fixed_entries: bool,
    /// If this is set to `true`, tasks can be scheduled on days where the user
    /// might be absent.
    ///
    /// Otherwise the scheduler will avoid scheduling tasks on those days.
    pub should_schedule_with_absences: bool,
    /// The maximum duration that can be scheduled on a single day.
    pub daily_limit: WorkingDuration,
}

impl Default for SchedulerOptions {
    fn default() -> Self {
        Self {
            should_schedule_with_fixed_entries: false,
            should_schedule_with_absences: false,
            daily_limit: working_duration!(06:00),
        }
    }
}
