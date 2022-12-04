use crate::input::toml_input::Task;
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
        mut dynamic_tasks: P,
        mut scheduler: S,
        fixed_scheduler: F,
    ) -> (Vec<(Id, ScheduledTime)>, Option<(Id, Task)>)
    where
        Id: Copy,
        P: Iterator<Item = (Id, Task)>,
        S: Scheduler,
        F: Fn(Date) -> WorkingDuration,
    {
        let mut result = Vec::new();

        // schedule fixed tasks in advance
        for date in self.start_date..=self.end_date {
            scheduler.schedule_in_advance(date, fixed_scheduler(date));
        }

        let mut current_task: Option<(Id, Task)> = None;
        for date in self.start_date..=self.end_date {
            let Some((id, task)) = current_task.as_ref().copied().or_else(|| {current_task = dynamic_tasks.next(); current_task}) else {
                break; // nothing to schedule
            };

            let mut possible_work_duration = scheduler.has_time_for(date, task.duration());

            // skips days where no work is possible
            if possible_work_duration == working_duration!(00:00) {
                continue;
            }

            let task_duration = task.duration();
            // if the task is longer than the possible work duration, we have to split it
            let worked_duration = min!(task_duration, possible_work_duration);

            result.push((id, ScheduledTime::new(date, worked_duration)));
            scheduler.schedule(date, worked_duration);

            possible_work_duration -= worked_duration;

            if worked_duration < task_duration {
                current_task = Some((id, task.with_duration(task_duration - worked_duration)));
            } else {
                current_task = None;
            }
        }

        (result, current_task)
    }
}
