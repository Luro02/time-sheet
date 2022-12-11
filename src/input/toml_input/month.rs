use indexmap::IndexMap;
use serde::Deserialize;

use crate::input::toml_input::{
    Absence, DynamicEntry, Entry, General, Holiday, Key, MultiEntry, Transfer,
};
use crate::time::Date;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EitherEntry {
    MultiEntry(MultiEntry),
    Entry(Entry),
}

impl IntoIterator for EitherEntry {
    type Item = Entry;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            EitherEntry::MultiEntry(multi_entry) => Box::new(multi_entry.into_iter()),
            EitherEntry::Entry(entry) => Box::new(std::iter::once(entry)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Month {
    general: General,
    transfer: Option<Transfer>,
    holiday: Option<Holiday>,
    #[serde(default)]
    entries: IndexMap<Key, EitherEntry>,
    // TODO: test this, so it crashes when missing? #[serde(default)] should be present
    dynamic: IndexMap<String, DynamicEntry>,
    #[serde(default)]
    absence: IndexMap<Key, Absence>,
}

impl Month {
    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn transfer(&self) -> Option<Transfer> {
        self.transfer
    }

    pub fn add_entries(&mut self, entries: impl IntoIterator<Item = (Key, EitherEntry)>) {
        self.entries.extend(entries);
    }

    pub fn entries(&self) -> impl Iterator<Item = (Key, Entry)> + '_ {
        self.entries
            .clone()
            .into_iter()
            .flat_map(|(k, v)| v.into_iter().map(move |v| (k.clone(), v)))
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = (&String, &DynamicEntry)> + '_ {
        self.dynamic.iter()
    }

    // TODO: use make_date?
    pub fn make_date(&self, day: usize) -> Date {
        Date::new(self.general.year(), self.general.month(), day).expect("failed to make date")
    }

    pub fn absences(&self) -> impl Iterator<Item = (Date, &Absence)> + '_ {
        self.absence
            .iter()
            .map(|(k, v)| (self.make_date(k.day()), v))
    }

    pub fn holiday(&self) -> Option<&Holiday> {
        self.holiday.as_ref()
    }
}
