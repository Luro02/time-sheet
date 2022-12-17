use serde::Deserialize;

use crate::input::toml_input::Key;
use crate::time::{TimeSpan, TimeStamp, WorkingDuration};
use crate::utils::MapEntry;

#[derive(Debug, Clone, Deserialize)]
pub struct MultiEntry {
    entries: Vec<Entry>,
}

impl MultiEntry {
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }
}

impl IntoIterator for MultiEntry {
    type Item = Entry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl From<Vec<Entry>> for MultiEntry {
    fn from(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl<'de> MapEntry<'de> for MultiEntry {
    type Key = <Entry as MapEntry<'de>>::Key;
    type Value = Self;

    fn new(key: Self::Key, value: Self::Value) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|entry| <Entry as MapEntry<'_>>::new(key.clone(), entry))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    // This is the key of the entry, will be added later
    #[serde(default)]
    key: Key,
    action: String,
    start: TimeStamp,
    end: TimeStamp,
    pause: Option<WorkingDuration>,
    is_vacation: Option<bool>,
    /// Can be used to introduce randomness around the specified date.
    ///
    /// For example if a `flex` of `"00:30"` is specified, then this program
    /// is allowed to randomly adjust this entrys start/end by +- 30mins.
    #[serde(default)]
    flex: WorkingDuration,
}

impl Entry {
    pub fn new(
        day: usize,
        action: String,
        span: TimeSpan,
        pause: Option<WorkingDuration>,
        is_vacation: Option<bool>,
    ) -> Self {
        Self {
            key: Key::from_day(day),
            action,
            start: span.start(),
            end: span.end(),
            pause,
            is_vacation,
            flex: WorkingDuration::default(),
        }
    }

    pub fn day(&self) -> usize {
        self.key.day()
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn start(&self) -> TimeStamp {
        self.start
    }

    pub fn end(&self) -> TimeStamp {
        self.end
    }

    pub fn pause(&self) -> Option<WorkingDuration> {
        self.pause
    }

    pub fn is_vacation(&self) -> bool {
        self.is_vacation.unwrap_or(false)
    }

    // TODO: make use of the flex
    pub fn flex(&self) -> WorkingDuration {
        self.flex
    }
}

impl<'de> MapEntry<'de> for Entry {
    type Key = Key;
    type Value = Entry;

    #[must_use]
    fn new(key: Self::Key, value: Self::Value) -> Self {
        let mut entry = value;
        entry.key = key;
        entry
    }
}
