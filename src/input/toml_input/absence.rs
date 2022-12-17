use serde::Deserialize;

use crate::input::toml_input::Key;
use crate::time::{TimeSpan, TimeStamp, WorkingDuration};
use crate::utils::MapEntry;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Absence {
    #[serde(default)]
    key: Key,
    /// When the absence starts on the first day.
    start: TimeStamp,
    /// When the absence ends on the last day.
    end: TimeStamp,
}

impl Absence {
    pub fn day(&self) -> usize {
        self.key.day()
    }

    #[must_use]
    pub const fn time_span(&self) -> TimeSpan {
        TimeSpan::new(self.start, self.end)
    }

    pub const fn duration(&self) -> WorkingDuration {
        self.time_span().duration()
    }
}

impl<'de> MapEntry<'de> for Absence {
    type Key = Key;
    type Value = Self;

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.key = key;
        value
    }
}
