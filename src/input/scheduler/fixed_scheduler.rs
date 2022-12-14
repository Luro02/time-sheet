use log::debug;

use crate::input::scheduler::{Scheduler, SchedulerOptions};
use crate::time::{Date, WorkingDuration};
use crate::working_duration;

#[derive(Clone, Debug, PartialEq)]
pub struct FixedScheduler<F> {
    f: F,
    should_mix: bool,
}

impl<F> FixedScheduler<F>
where
    F: Fn(Date) -> WorkingDuration,
{
    #[must_use]
    pub const fn new(f: F, options: &SchedulerOptions) -> Self {
        Self {
            f,
            should_mix: options.should_schedule_with_fixed_entries,
        }
    }
}

impl<F> Scheduler for FixedScheduler<F>
where
    F: Fn(Date) -> WorkingDuration,
{
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        let fixed_work = (self.f)(date);

        if !self.should_mix && fixed_work > working_duration!(00:00) {
            working_duration!(00:00)
        } else {
            let result = {
                if wanted_duration < fixed_work {
                    wanted_duration
                } else {
                    wanted_duration - fixed_work
                }
            };

            debug!(
                "FixedScheduler({}, {}): can schedule at most {}",
                date, fixed_work, result
            );

            result
        }
    }
}
