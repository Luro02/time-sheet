use serde::{Deserialize, Serialize};

use crate::input::json_input::Entry;
use crate::input::toml_input::{self, Transfer};
use crate::time::{Month, WorkingDuration, Year};

fn default_schema() -> &'static str {
    "https://raw.githubusercontent.com/kit-sdq/TimeSheetGenerator/master/examples/schemas/month.json"
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonthFile {
    #[serde(rename = "$schema")]
    pub(in crate::input) schema: String,
    pub(in crate::input) year: Year,
    pub(in crate::input) month: Month,
    pub(in crate::input) pred_transfer: WorkingDuration,
    pub(in crate::input) succ_transfer: WorkingDuration,
    pub(in crate::input) entries: Vec<Entry>,
}

impl From<(WorkingDuration, toml_input::Month)> for MonthFile {
    fn from((working_duration, month): (WorkingDuration, toml_input::Month)) -> Self {
        Self {
            schema: default_schema().to_string(),
            year: month.general().year(),
            month: month.general().month(),
            pred_transfer: month
                .transfer()
                .map_or_else(Default::default, |t| t.previous()),
            succ_transfer: month.transfer().map_or_else(Default::default, |t| t.next()),
            entries: month
                .entries(working_duration)
                .map(|(key, entry)| Entry::from((key.clone(), entry.clone())))
                .collect(),
        }
    }
}

impl MonthFile {
    pub(in crate::input) fn default_schema() -> &'static str {
        default_schema()
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
