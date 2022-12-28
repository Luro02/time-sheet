use std::fmt;

use log::debug;

use crate::input::strategy::Strategy;
use crate::input::Task;
use crate::time::Date;

/// The tasks are scheduled in the order they are given and until
/// their entire time is used up.
///
/// This might result in some tasks never being scheduled.
pub struct FirstComeFirstServe<Id> {
    tasks: Vec<(Id, Task)>,
}

impl<Id> FirstComeFirstServe<Id> {
    /// Creates a new instance with the provided tasks.
    ///
    /// The tasks are scheduled in the order they are given.
    #[must_use]
    pub fn new(mut tasks: Vec<(Id, Task)>) -> Self {
        tasks.reverse();
        Self { tasks }
    }
}

impl<Id> Strategy<Id> for FirstComeFirstServe<Id>
where
    Id: fmt::Debug + Clone,
{
    fn peek_task(&self, date: Date) -> Option<(&Id, &Task)> {
        debug!("[fcfs] peeked task for date `{}`", date);
        self.tasks.last().map(|(id, task)| (id, task))
    }

    fn next_task(&mut self, _date: Date) -> Option<(Id, Task)> {
        // NOTE: date does not matter for now (until repeating entries can be dynamic).

        debug!(
            "[fcfs] requested next task, returning task with id `{:?}`",
            self.tasks.last().map(|(id, _)| id)
        );
        // pop is more efficient than removing the first element
        // so the tasks are reversed in the constructor
        self.tasks.pop()
    }

    fn push_task(&mut self, id: Id, task: Task) {
        debug!(
            "[fcfs] pushed task with id `{:?}`, remaining duration: {}",
            id,
            task.duration()
        );
        self.tasks.push((id, task));
    }

    fn to_remaining(&self) -> Vec<(Id, Task)> {
        self.tasks.clone()
    }
}
