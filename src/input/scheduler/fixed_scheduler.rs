use crate::input::scheduler::Scheduler;
use crate::time::{Date, WorkingDuration};
use crate::working_duration;

pub struct FixedScheduler<F> {
    f: F,
    should_mix: bool,
}

impl<F> FixedScheduler<F>
where
    F: Fn(Date) -> WorkingDuration,
{
    #[must_use]
    pub const fn new(f: F, should_mix: bool) -> Self {
        Self { f, should_mix }
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
        } else if wanted_duration < fixed_work {
            working_duration!(00:00)
        } else {
            wanted_duration - fixed_work
        }
    }
}
