use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::files::Entry;
use crate::time::{Date, Month, WorkingDuration, Year};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonthFile {
    #[serde(rename = "$schema")]
    schema: String,
    year: Year,
    month: Month,
    pred_transfer: WorkingDuration,
    succ_transfer: WorkingDuration,
    entries: Vec<Entry>,
    #[serde(skip_serializing)]
    working_time: Option<WorkingDuration>,
}

impl MonthFile {
    #[must_use]
    pub fn year(&self) -> Year {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> Month {
        self.month
    }

    #[must_use]
    pub fn succ_transfer(&self) -> Duration {
        // TODO: why only succ_transfer?
        self.succ_transfer.into()
    }

    #[must_use]
    pub fn total_time(&self) -> Duration {
        let mut result = Duration::from_secs(0);

        for entry in self.entries.iter() {
            result += entry.work_duration();
        }

        result
    }

    /// Returns the amount of time that the user should have worked in this month.
    ///
    /// For example if the user has to work 40 hours a month, then there will be
    /// a working time of 40 hours returned.
    #[must_use]
    pub fn working_time(&self) -> Option<WorkingDuration> {
        self.working_time
    }

    pub fn days(&self) -> impl Iterator<Item = Date> + '_ {
        self.entries.iter().map(|entry| {
            Date::new(self.year(), self.month(), entry.day()).expect("the date is invalid???")
        })
    }

    pub fn entries_on_day(&self, date: Date) -> impl Iterator<Item = &Entry> + '_ {
        self.entries
            .iter()
            .filter(move |entry| entry.day() == date.day())
    }
}