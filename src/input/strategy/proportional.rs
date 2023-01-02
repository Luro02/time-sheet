use std::fmt;

use crate::input::strategy::{FirstComeFirstServe, Strategy};
use crate::input::Task;
use crate::time::{Date, WorkingDuration};
use crate::utils;

pub struct Proportional<Id> {
    inner: FirstComeFirstServe<Id>,
}

impl<Id> Proportional<Id> {
    /// Creates a new instance with the provided tasks.
    ///
    /// The tasks are scheduled in the order they are given.
    #[must_use]
    pub fn new(tasks: Vec<(Id, Task)>, remaining_time: WorkingDuration) -> Self {
        let mut input_tasks = tasks
            .iter()
            .map(|(_, t)| t.duration().as_mins() as usize)
            .collect::<Vec<_>>();

        let remainder =
            utils::divide_proportionally(remaining_time.as_mins() as usize, &mut input_tasks);

        let middle = input_tasks.len() / 2;
        let tasks = tasks
            .into_iter()
            .enumerate()
            .map(|(i, (id, task))| {
                let mut duration = WorkingDuration::from_mins(input_tasks[i] as u16);

                if i == middle {
                    duration += WorkingDuration::from_mins(remainder as u16);
                }

                (id, task.with_duration(duration))
            })
            .collect::<Vec<_>>();

        Self {
            inner: FirstComeFirstServe::new(tasks),
        }
    }
}

impl<Id> Strategy<Id> for Proportional<Id>
where
    Id: fmt::Debug + Clone,
{
    fn next_task(&mut self, date: Date) -> Option<(Id, Task)> {
        self.inner.next_task(date)
    }

    fn push_task(&mut self, id: Id, task: Task) {
        self.inner.push_task(id, task)
    }

    fn to_remaining(&self) -> Vec<(Id, Task)> {
        self.inner.to_remaining()
    }
}
