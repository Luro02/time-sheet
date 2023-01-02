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

    fn next_task_position(&self, date: Date) -> Option<usize> {
        // prioritize tasks that do apply on specific dates only:
        if let Some((pos, _)) = self
            .tasks
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, (_, t))| t.applies_on(date) && t.has_filter())
            .next()
        {
            return Some(pos);
        }

        self.tasks
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, (_, t))| t.applies_on(date))
            .map(|(i, _)| i)
            .next()
    }
}

impl<Id> Strategy<Id> for FirstComeFirstServe<Id>
where
    Id: fmt::Debug + Clone,
{
    fn next_task(&mut self, date: Date) -> Option<(Id, Task)> {
        if let Some(next_task_position) = self.next_task_position(date) {
            let (id, task) = self.tasks.remove(next_task_position);
            debug!("requested next task, returning task with id `{:?}`", &id);
            return Some((id, task));
        }

        None
    }

    fn push_task(&mut self, id: Id, task: Task) {
        debug!(
            "pushed task with id `{:?}`, remaining duration: {}",
            id,
            task.duration()
        );
        self.tasks.push((id, task));
    }

    fn to_remaining(&self) -> Vec<(Id, Task)> {
        self.tasks.clone()
    }
}
