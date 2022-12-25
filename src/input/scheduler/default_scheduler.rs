use crate::input::scheduler::{
    AbsenceScheduler, DailyLimiter, FixedScheduler, MonthScheduler, Scheduler, SchedulerOptions,
    WorkdayScheduler,
};
use crate::input::Month;
use crate::input::Transfer;
use crate::time::{Date, WorkingDuration};
use crate::working_duration;

#[derive(Debug, Clone)]
pub struct DefaultScheduler<F> {
    scheduler: (
        WorkdayScheduler,
        FixedScheduler<F>,
        AbsenceScheduler<F>,
        DailyLimiter,
    ),
    month_scheduler: MonthScheduler,
}

impl<'a> DefaultScheduler<Box<dyn Fn(Date) -> WorkingDuration + 'a>> {
    #[must_use]
    pub fn new(month: &'a Month, options: &SchedulerOptions) -> Self {
        Self {
            scheduler: (
                WorkdayScheduler::new(),
                FixedScheduler::new(Box::new(|date| month.working_time_on_day(date)), options),
                AbsenceScheduler::new(Box::new(|date| month.absence_time_on_day(date)), options),
                DailyLimiter::new(options),
            ),
            month_scheduler: MonthScheduler::new_with_available_time(
                month.year(),
                month.month(),
                month.expected_working_duration(),
                |date| {
                    if date.is_workday() {
                        options
                            .daily_limit
                            .saturating_sub(month.absence_time_on_day(date))
                    } else {
                        working_duration!(00:00)
                    }
                },
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
