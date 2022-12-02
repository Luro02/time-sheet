use std::time::Duration;

use indexmap::IndexMap;
use serde::ser;
use serde::Serialize;

use crate::input::json_input::{Entry, MonthFile};
use crate::input::toml_input::{DynamicEntry, Transfer};
use crate::time::{self, Date, WeekDay, WorkingDuration, Year};
use crate::{time_stamp, working_duration};

#[derive(Debug, Clone)]
pub struct Month {
    year: Year,
    month: time::Month,
    dynamic_entries: IndexMap<String, DynamicEntry>,
    expected_working_duration: Option<WorkingDuration>,
    transfer: Transfer,
    entries: Vec<Entry>,
}

impl Month {
    const MAXIMUM_WORK_DURATION: WorkingDuration = working_duration!(08:00);

    #[must_use]
    pub fn new(
        month: time::Month,
        year: Year,
        transfer: Transfer,
        entries: Vec<Entry>,
        dynamic_entries: IndexMap<String, DynamicEntry>,
        expected_working_duration: Option<WorkingDuration>,
    ) -> Self {
        Self {
            month,
            year,
            transfer,
            entries,
            dynamic_entries,
            expected_working_duration,
        }
    }

    /// Returns the amount of time that the user should have worked in this month.
    ///
    /// For example if the user has to work 40 hours a month, then there will be
    /// a working time of 40 hours returned.
    #[must_use]
    pub fn expected_working_duration(&self) -> WorkingDuration {
        self.expected_working_duration
            .unwrap_or(working_duration!(40:00))
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = (&String, &DynamicEntry)> {
        self.dynamic_entries.iter()
    }

    #[must_use]
    pub fn year(&self) -> Year {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> time::Month {
        self.month
    }

    #[must_use]
    pub fn total_working_time(&self) -> Duration {
        let mut result = Duration::from_secs(0);

        for entry in self.entries.iter() {
            result += entry.work_duration();
        }

        // add the time from the previous/next month
        result + self.transfer
    }

    /// Checks if one can work the provided `duration` on that `date`, without exceeding
    /// the maximum working time on that week/day/month.
    pub fn exceeds_working_duration_on_with(&self, date: Date, duration: WorkingDuration) -> bool {
        let time_on_day: WorkingDuration = self
            .entries_on_day(date)
            .map(|e| e.work_duration())
            .sum::<WorkingDuration>()
            // add the provided duration to the total working duration
            + duration;

        time_on_day > self.maximum_work_duration()
    }

    #[must_use]
    pub const fn maximum_work_duration(&self) -> WorkingDuration {
        Self::MAXIMUM_WORK_DURATION
    }

    pub fn days_with_time_for(&self, duration: WorkingDuration) -> impl Iterator<Item = Date> + '_ {
        self.year()
            .iter_days_in(self.month())
            .filter(move |date| !self.exceeds_working_duration_on_with(*date, duration))
    }

    /// Checks whether or not there is work scheduled on the provided date.
    #[must_use]
    pub fn has_entries_on(&self, date: Date) -> bool {
        self.entries_on_day(date).next().is_some()
    }

    /// Returns days in the `month` where no fixed work is planned or has been made.
    pub fn free_days(&self) -> impl Iterator<Item = Date> + '_ {
        self.year()
            .iter_days_in(self.month())
            // skip all sundays, where one is not allowed to work
            .filter(|date| date.week_day() != WeekDay::Sunday)
            // skip all holidays, where one is not allowed to work
            .filter(|date| !date.is_holiday())
            // skip all days where there is already an entry
            .filter(|date| self.entries_on_day(*date).next().is_some())
    }

    /// Returns an iterator over all entries that are on the given day.
    pub fn entries_on_day(&self, date: Date) -> impl Iterator<Item = &Entry> + '_ {
        self.entries
            .iter()
            .filter(move |entry| entry.day() == date.day())
    }

    /// Returns the transfer time for the month.
    /// (how much time is transfered to the next month/from the previous month)
    #[must_use]
    pub fn transfer(&self) -> &Transfer {
        &self.transfer
    }

    fn to_month_file(&self) -> MonthFile {
        let mut entries = self.entries.clone();

        let mut mapping = Vec::with_capacity(self.dynamic_entries.len());
        let mut durations = Vec::with_capacity(mapping.capacity());

        for (action, dynamic_entry) in self.dynamic_entries() {
            if let Some(duration) = dynamic_entry.duration() {
                mapping.push((action, dynamic_entry));
                durations.push((mapping.len() - 1, duration));
            }
        }

        let (_transfer, results, _transfer_tasks) =
            DynamicEntry::distribute_fixed(durations.into_iter(), self);

        // TODO: what to do with the transfer_tasks and transfer?

        for (id, date, duration) in results {
            let (action, _dynamic_entry) = mapping[id];
            let mut pause = None;

            // TODO: make it possible to configure this?
            // TODO: automagically add pauses for too long fixed entries?
            if duration > working_duration!(04:00) {
                pause = Some(working_duration!(00:30));
            }

            let start = time_stamp!(10:00);
            let end = start + duration + pause.unwrap_or_default();

            entries.push(Entry::new(
                action.to_string(),
                date.day(),
                start,
                end,
                pause.map(Into::into),
                None, // TODO: add vacation?
            ))
        }

        // sort the entries in the json file, so that no problems occur with the java tool
        entries.sort();

        MonthFile {
            schema: MonthFile::default_schema().to_string(),
            year: self.year,
            month: self.month,
            pred_transfer: self.transfer().previous(),
            succ_transfer: self.transfer().next(),
            entries,
        }
    }
}

impl Serialize for Month {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_month_file().serialize(serializer)
    }
}
