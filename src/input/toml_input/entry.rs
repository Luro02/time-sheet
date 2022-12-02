use serde::Deserialize;

use crate::time::{TimeStamp, WorkingDuration};

#[derive(Debug, Clone, Deserialize)]
pub struct MultiEntry {
    entries: Vec<Entry>,
}

impl MultiEntry {
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    action: String,
    start: TimeStamp,
    end: TimeStamp,
    pause: Option<TimeStamp>,
    is_vacation: Option<bool>,
    /// Can be used to introduce randomness around the specified date.
    ///
    /// For example if a `flex` of `"00:30"` is specified, then this program
    /// is allowed to randomly adjust this entrys start/end by +- 30mins.
    #[serde(default)]
    flex: WorkingDuration,
}

impl Entry {
    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn start(&self) -> &TimeStamp {
        &self.start
    }

    pub fn end(&self) -> &TimeStamp {
        &self.end
    }

    pub fn pause(&self) -> Option<&TimeStamp> {
        self.pause.as_ref()
    }

    pub fn is_vacation(&self) -> bool {
        self.is_vacation.unwrap_or(false)
    }

    pub fn flex(&self) -> &WorkingDuration {
        &self.flex
    }
}
