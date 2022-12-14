use crate::input::scheduler::{Scheduler, SchedulerOptions};
use crate::time::{Date, WorkingDuration};
use crate::{min, working_duration};

/// A scheduler that prevents too much work being scheduled on
/// dates where the user is absent.
#[derive(Clone, Debug, PartialEq)]
pub struct AbsenceScheduler<F> {
    f: F,
    should_mix: bool,
    limit: WorkingDuration,
}

impl<F> AbsenceScheduler<F>
where
    F: Fn(Date) -> WorkingDuration,
{
    #[must_use]
    pub const fn new(f: F, options: &SchedulerOptions) -> Self {
        Self {
            f,
            should_mix: options.should_schedule_with_absences,
            limit: options.daily_limit,
        }
    }
}

impl<F> Scheduler for AbsenceScheduler<F>
where
    F: Fn(Date) -> WorkingDuration,
{
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        let absent_duration = (self.f)(date);

        // the min prevents an underflow, when absent_duration > self.limit
        let remaining = self.limit - min!(self.limit, absent_duration);

        if !self.should_mix && absent_duration > working_duration!(00:00) {
            working_duration!(00:00)
        } else {
            min!(remaining, wanted_duration)
        }
    }
}
