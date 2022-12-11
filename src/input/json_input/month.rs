use serde::{Deserialize, Serialize};

use crate::input::json_input::Entry;
use crate::input::toml_input::{self, Transfer};
use crate::time::{Month, WorkingDuration, Year};

const fn default_schema() -> &'static str {
    "https://raw.githubusercontent.com/kit-sdq/TimeSheetGenerator/master/examples/schemas/month.json"
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MonthFile {
    #[serde(rename = "$schema")]
    schema: String,
    year: Year,
    month: Month,
    pred_transfer: WorkingDuration,
    succ_transfer: WorkingDuration,
    entries: Vec<Entry>,
}

impl From<toml_input::Month> for MonthFile {
    fn from(month: toml_input::Month) -> Self {
        Self::new(
            month.general().year(),
            month.general().month(),
            month.transfer().unwrap_or_default(),
            month
                .entries()
                .map(|(key, entry)| Entry::from((key.clone(), entry.clone())))
                .collect(),
        )
    }
}

impl MonthFile {
    pub fn new(year: Year, month: Month, transfer: Transfer, entries: Vec<Entry>) -> Self {
        Self {
            schema: default_schema().to_string(),
            year,
            month,
            pred_transfer: transfer.previous(),
            succ_transfer: transfer.next(),
            entries,
        }
    }

    #[must_use]
    pub fn year(&self) -> Year {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> Month {
        self.month
    }

    pub fn transfer(&self) -> Transfer {
        Transfer::new(self.pred_transfer, self.succ_transfer)
    }

    pub(in crate::input) fn into_entries(self) -> Vec<Entry> {
        self.entries
    }
}
