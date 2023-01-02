use crate::input::strategy::{PeekableStrategy, Strategy};
use crate::input::Scheduler;
use crate::time::{Date, WorkingDuration};
use crate::{min, working_duration};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkSchedule {
    /// The start date of the work schedule.
    start_date: Date,
    /// The end date of the work schedule (inclusive)
    end_date: Date,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTime {
    duration: WorkingDuration,
    date: Date,
}

impl ScheduledTime {
    #[must_use]
    pub fn new(date: Date, duration: WorkingDuration) -> Self {
        Self { date, duration }
    }

    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    #[must_use]
    pub fn duration(&self) -> WorkingDuration {
        self.duration
    }
}

impl WorkSchedule {
    #[must_use]
    pub(crate) fn new(start_date: Date, end_date: Date) -> Self {
        assert_eq!(start_date.year(), end_date.year());
        assert_eq!(start_date.month(), end_date.month());

        Self {
            start_date,
            end_date,
        }
    }

    pub fn schedule<S, P, Id, F>(
        &self,
        strategy: &mut PeekableStrategy<Id, P>,
        mut scheduler: S,
        fixed_scheduler: F,
    ) -> Vec<(Id, ScheduledTime)>
    where
        Id: Copy,
        P: Strategy<Id>,
        S: Scheduler,
        F: Fn(Date) -> WorkingDuration,
    {
        let mut result = Vec::new();

        // schedule fixed tasks in advance
        for date in self.start_date..=self.end_date {
            scheduler.schedule_in_advance(date, fixed_scheduler(date));
        }

        for date in self.start_date..=self.end_date {
            let Some((_, task)) = strategy.peek_task(date) else {
                continue; // nothing to schedule
                // TODO: might be a good idea to ask the strategy if there
                // are any tasks left at all and quit if there are none remaining
            };

            let mut possible_work_duration = scheduler.has_time_for(date, task.duration());

            if task.can_bypass_weekly_limit() {
                // if the task can bypass the weekly limit, we can schedule it
                // even if the weekly limit is reached
                //
                // TODO: this will be problematic for tasks that are way too long
                // TODO: can result in daily limits being exceeded as well as conflicts
                //       with fixed entries
                possible_work_duration = task.duration();
            }

            // skips days where no work is possible
            if possible_work_duration == working_duration!(00:00) {
                continue;
            }

            let task_duration = task.duration();
            // if the task is longer than the possible work duration, we have to split it
            let worked_duration = min!(task_duration, possible_work_duration);

            // consume the task only when it will definitely be scheduled
            let (id, task) = strategy.next_task(date).unwrap();

            result.push((id, ScheduledTime::new(date, worked_duration)));
            scheduler.schedule(date, worked_duration);

            possible_work_duration -= worked_duration;

            // only reschedule the task if it is not finished yet:
            if worked_duration < task_duration {
                strategy.push_task(id, task.with_duration(task_duration - worked_duration));
            }
        }

        result
    }
}
