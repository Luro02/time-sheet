//! This module contains a trait and implementations for different strategies
//! that decide when and where a task should be scheduled.

mod first_come_first_serve;
mod proportional;

pub use first_come_first_serve::*;
pub use proportional::*;

use std::ops::{Deref, DerefMut};

use crate::input::Task;
use crate::time::Date;

pub trait Strategy<Id> {
    /// Returns the next task that should be scheduled on that date
    /// without removing it.
    #[must_use]
    fn peek_task(&self, date: Date) -> Option<(&Id, &Task)>;

    /// Returns the next task that should be scheduled.
    ///
    /// The `date` is provided as a suggestion on which date
    /// the task might be scheduled.
    #[must_use]
    fn next_task(&mut self, date: Date) -> Option<(Id, Task)>;

    /// Adds the task back, so that it can be scheduled again
    /// if needed.
    ///
    /// It is not required that the task is added again if the
    /// strategy does not need it.
    ///
    /// # Note
    ///
    /// The task might have a different working duration than
    /// the was returned by `next_task`.
    fn push_task(&mut self, id: Id, task: Task);

    /// Returns the remaining tasks.
    #[must_use]
    fn to_remaining(&self) -> Vec<(Id, Task)>;
}

impl<Id, S> Strategy<Id> for &mut S
where
    S: Strategy<Id>,
{
    fn peek_task(&self, date: Date) -> Option<(&Id, &Task)> {
        <S as Strategy<Id>>::peek_task(*self, date)
    }

    fn next_task(&mut self, date: Date) -> Option<(Id, Task)> {
        <S as Strategy<Id>>::next_task(*self, date)
    }

    fn push_task(&mut self, id: Id, task: Task) {
        <S as Strategy<Id>>::push_task(*self, id, task)
    }

    fn to_remaining(&self) -> Vec<(Id, Task)> {
        <S as Strategy<Id>>::to_remaining(*self)
    }
}

impl<Id> Strategy<Id> for Box<dyn Strategy<Id>> {
    fn peek_task(&self, date: Date) -> Option<(&Id, &Task)> {
        Box::deref(self).peek_task(date)
    }

    fn next_task(&mut self, date: Date) -> Option<(Id, Task)> {
        Box::deref_mut(self).next_task(date)
    }

    fn push_task(&mut self, id: Id, task: Task) {
        Box::deref_mut(self).push_task(id, task)
    }

    fn to_remaining(&self) -> Vec<(Id, Task)> {
        Box::deref(self).to_remaining()
    }
}
