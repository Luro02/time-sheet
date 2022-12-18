use serde::{Deserialize, Serialize};

use crate::input::WorkingArea;
use crate::time::{Date, WorkingDuration};
use crate::utils::{self, MapEntry};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contract {
    #[serde(default)]
    department: String,
    working_time: WorkingDuration,
    area: WorkingArea,
    wage: Option<f32>,
    #[serde(with = "utils::serde_toml_local_date")]
    start_date: Date,
    #[serde(with = "utils::serde_toml_local_date")]
    end_date: Date,
    bg_content: Option<String>,
}

impl Contract {
    /// The department of the contract.
    pub fn department(&self) -> &str {
        &self.department
    }

    /// How long the employee has to work each month.
    pub fn expected_working_duration(&self) -> WorkingDuration {
        self.working_time
    }

    /// In which field the employee is working at the university.
    pub fn working_area(&self) -> WorkingArea {
        self.area
    }

    /// How much the employee makes per hour (in euros).
    pub fn wage(&self) -> Option<f32> {
        self.wage
    }

    /// When the contract starts.
    pub fn start_date(&self) -> Date {
        self.start_date
    }

    /// When the contract ends.
    pub fn end_date(&self) -> Date {
        self.end_date
    }

    /// In the bottom left of the final PDF is a small signature.
    ///
    /// If this is set, then the signature will be replaced with this text.
    pub fn bg_content(&self) -> Option<&str> {
        self.bg_content.as_deref()
    }
}

impl<'de> MapEntry<'de> for Contract {
    type Key = String;
    type Value = Self;

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.department = key;
        value
    }
}
