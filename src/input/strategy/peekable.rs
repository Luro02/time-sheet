use crate::input::strategy::{Strategy, Task};
use crate::time::Date;

pub struct PeekableStrategy<Id, S> {
    strategy: S,
    peeked: Option<(Id, Task)>,
}

impl<Id, S> PeekableStrategy<Id, S>
where
    S: Strategy<Id>,
{
    pub fn new(strategy: S) -> Self {
        Self {
            strategy,
            peeked: None,
        }
    }

    #[must_use]
    pub fn peek_task(&mut self, date: Date) -> Option<(&Id, &Task)> {
        if self.peeked.is_none() {
            self.peeked = self.strategy.next_task(date);
        }

        self.peeked.as_ref().map(|(id, task)| (id, task))
    }
}

impl<Id, S> Strategy<Id> for PeekableStrategy<Id, S>
where
    S: Strategy<Id>,
    Id: Clone,
{
    fn next_task(&mut self, date: Date) -> Option<(Id, Task)> {
        if let Some((id, task)) = self.peeked.take() {
            Some((id, task))
        } else {
            self.strategy.next_task(date)
        }
    }

    fn push_task(&mut self, id: Id, task: Task) {
        if let Some((id, task)) = self.peeked.take() {
            self.strategy.push_task(id, task);
        }

        self.strategy.push_task(id, task);
    }

    fn to_remaining(&self) -> Vec<(Id, Task)> {
        let mut result = self.strategy.to_remaining();

        if let Some((id, task)) = self.peeked.as_ref() {
            result.push((id.clone(), task.clone()));
        }

        result
    }
}
