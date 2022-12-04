use crate::input::scheduler::{
    DailyLimiter, FixedScheduler, MonthScheduler, Scheduler, WorkdayScheduler,
};
use crate::input::Month;
use crate::input::Transfer;
use crate::time::{Date, WorkingDuration};

#[derive(Debug, Clone)]
pub struct DefaultScheduler<F> {
    scheduler: (WorkdayScheduler, FixedScheduler<F>, DailyLimiter),
    month_scheduler: MonthScheduler,
}

impl<'a> DefaultScheduler<Box<dyn Fn(Date) -> WorkingDuration + 'a>> {
    #[must_use]
    pub fn new(month: &'a Month) -> Self {
        Self {
            scheduler: (
                WorkdayScheduler::new(),
                FixedScheduler::new(
                    Box::new(|date| {
                        month
                            .entries_on_day(date)
                            .map(|e| e.work_duration())
                            .sum::<WorkingDuration>()
                    }),
                    false,
                ),
                DailyLimiter::default(),
            ),
            month_scheduler: MonthScheduler::new(
                month.year(),
                month.month(),
                month.expected_working_duration(),
            ),
        }
    }
}

impl<F> DefaultScheduler<F> {
    #[must_use]
    pub fn transfer_time(&self) -> Transfer {
        self.month_scheduler.transfer_time()
    }
}

impl<F> Scheduler for DefaultScheduler<F>
where
    F: Fn(Date) -> WorkingDuration,
{
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        let result = self.scheduler.has_time_for(date, wanted_duration);
        self.month_scheduler.has_time_for(date, result)
    }

    fn schedule(&mut self, date: Date, worked: WorkingDuration) {
        self.scheduler.schedule(date, worked);
        self.month_scheduler.schedule(date, worked);
    }

    fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
        self.scheduler.schedule_in_advance(date, worked);
        self.month_scheduler.schedule_in_advance(date, worked);
    }
}
