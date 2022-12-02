use crate::input::scheduler::Scheduler;
use crate::input::toml_input::Transfer;
use crate::time::{Date, WorkingDuration};
use crate::working_duration;

#[derive(Debug, Clone, PartialEq)]
pub struct TimeSpanScheduler {
    start_date: Date,
    end_date: Date,
    available_duration: WorkingDuration,
    transfer_time: WorkingDuration,
}

impl TimeSpanScheduler {
    pub fn new(start_date: Date, end_date: Date, available_duration: WorkingDuration) -> Self {
        Self {
            start_date,
            end_date,
            available_duration,
            transfer_time: working_duration!(00:00),
        }
    }

    #[must_use]
    pub const fn transfer_time(&self) -> WorkingDuration {
        self.transfer_time
    }

    #[must_use]
    pub const fn remaining_time(&self) -> WorkingDuration {
        self.available_duration
    }

    #[must_use]
    pub const fn transfer(&self) -> Transfer {
        Transfer::new(self.remaining_time(), self.transfer_time)
    }

    fn sub_remaining_time(&mut self, worked: WorkingDuration) {
        if self.available_duration >= worked {
            self.available_duration -= worked;
        } else {
            self.transfer_time += worked - self.available_duration;
            self.available_duration = working_duration!(00:00);
        }
    }

    fn add_remaining_time(&mut self, not_worked: WorkingDuration) {
        let mut remainder = working_duration!(00:00);

        if self.transfer_time >= not_worked {
            self.transfer_time -= not_worked;
        } else {
            remainder = not_worked - self.transfer_time;
            self.transfer_time = working_duration!(00:00);
        }

        self.available_duration += remainder;
    }

    pub fn add_transfer(&mut self, transfer: Transfer) {
        self.add_remaining_time(transfer.previous());
        self.sub_remaining_time(transfer.next());
    }

    #[must_use]
    pub fn take_transfer(&mut self) -> Transfer {
        let transfer = self.transfer();
        self.transfer_time = working_duration!(00:00);
        self.available_duration = working_duration!(00:00);

        transfer
    }
}

impl Scheduler for TimeSpanScheduler {
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        // ignore dates outside of the time span
        if date < self.start_date || date > self.end_date {
            wanted_duration
        } else if wanted_duration > self.available_duration {
            self.available_duration
        } else {
            wanted_duration
        }
    }

    fn schedule(&mut self, date: Date, worked: WorkingDuration) {
        if date < self.start_date || date > self.end_date {
            return;
        }

        self.sub_remaining_time(worked);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::date;

    #[test]
    fn test_add_transfer() {
        let mut scheduler = TimeSpanScheduler::new(
            date!(2022:11:01),
            date!(2022:11:30),
            working_duration!(08:05),
        );

        scheduler.add_transfer(Transfer::new(
            working_duration!(00:00),
            working_duration!(00:00),
        ));

        assert_eq!(
            scheduler.transfer(),
            Transfer::new(working_duration!(08:05), working_duration!(00:00))
        );

        scheduler.add_transfer(Transfer::new(
            working_duration!(01:00),
            working_duration!(00:00),
        ));

        assert_eq!(
            scheduler.transfer(),
            Transfer::new(working_duration!(09:05), working_duration!(00:00))
        );

        scheduler.add_transfer(Transfer::new(
            working_duration!(02:00),
            working_duration!(03:00),
        ));

        assert_eq!(
            scheduler.transfer(),
            Transfer::new(working_duration!(08:05), working_duration!(00:00))
        );

        scheduler.add_transfer(Transfer::new(
            working_duration!(09:00),
            working_duration!(20:00),
        ));

        assert_eq!(
            scheduler.transfer(),
            Transfer::new(working_duration!(00:00), working_duration!(02:55))
        );
    }
}
