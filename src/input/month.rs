use std::collections::HashMap;
use std::time::Duration;

use serde::ser;
use serde::Serialize;

use crate::input::json_input::{Entry, MonthFile};
use crate::input::toml_input::{DynamicEntry, Transfer};
use crate::time::{self, Date, WorkingDuration, Year};
use crate::working_duration;

#[derive(Debug, Clone)]
pub struct Month {
    year: Year,
    month: time::Month,
    dynamic_entries: HashMap<String, DynamicEntry>,
    expected_working_duration: Option<WorkingDuration>,
    transfer: Transfer,
    entries: Vec<Entry>,
}

impl Month {
    #[must_use]
    pub(super) fn new(
        month: time::Month,
        year: Year,
        transfer: Transfer,
        entries: Vec<Entry>,
        dynamic_entries: HashMap<String, DynamicEntry>,
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
        result + self.transfer.clone()
    }

    /// Returns an iterator over all days in the month that have an entry.
    // TODO: should this deduplicate days?
    pub fn days(&self) -> impl Iterator<Item = Date> + '_ {
        self.entries.iter().map(|entry| {
            Date::new(self.year(), self.month(), entry.day()).expect("the date is invalid???")
        })
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
        // TODO: add the dynamic entries to the entries!
        MonthFile {
            schema: MonthFile::default_schema().to_string(),
            year: self.year,
            month: self.month,
            pred_transfer: *self.transfer().previous_month(),
            succ_transfer: *self.transfer().next_month(),
            entries: self.entries.clone(),
            working_time: Some(self.expected_working_duration()),
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
