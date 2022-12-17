use serde::Deserialize;

use crate::input::toml_input::{
    Absence, DynamicEntry, Entry, General, Holiday, MultiEntry, Transfer,
};
use crate::time::Date;
use crate::utils::{self, MapEntry};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum EitherEntry {
    MultiEntry(MultiEntry),
    Entry(Entry),
}

impl IntoIterator for EitherEntry {
    type Item = Entry;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::MultiEntry(multi_entry) => Box::new(multi_entry.into_iter()),
            Self::Entry(entry) => Box::new(std::iter::once(entry)),
        }
    }
}

impl<'a> IntoIterator for &'a EitherEntry {
    type Item = &'a Entry;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            EitherEntry::MultiEntry(multi_entry) => Box::new(multi_entry.iter()),
            EitherEntry::Entry(entry) => Box::new(std::iter::once(entry)),
        }
    }
}

impl<'de> MapEntry<'de> for EitherEntry {
    type Key = <Entry as MapEntry<'de>>::Key;
    type Value = Self;

    fn new(key: Self::Key, value: Self::Value) -> Self {
        match value {
            Self::MultiEntry(value) => {
                Self::MultiEntry(<MultiEntry as MapEntry<'_>>::new(key, value))
            }
            Self::Entry(value) => Self::Entry(<Entry as MapEntry<'_>>::new(key, value)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Month {
    general: General,
    transfer: Option<Transfer>,
    holiday: Option<Holiday>,
    #[serde(default, deserialize_with = "utils::deserialize_map_entry")]
    entries: Vec<EitherEntry>,
    #[serde(default, deserialize_with = "utils::deserialize_map_entry")]
    dynamic: Vec<DynamicEntry>,
    #[serde(default, deserialize_with = "utils::deserialize_map_entry")]
    absence: Vec<Absence>,
}

impl Month {
    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn transfer(&self) -> Option<Transfer> {
        self.transfer
    }

    pub fn add_entries(&mut self, entries: impl IntoIterator<Item = Entry>) {
        self.entries
            .extend(entries.into_iter().map(EitherEntry::Entry));
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> + '_ {
        self.entries.iter().flatten()
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = &DynamicEntry> + '_ {
        self.dynamic.iter()
    }

    fn make_date(&self, day: usize) -> Date {
        Date::new(self.general.year(), self.general.month(), day).expect("failed to make date")
    }

    pub fn absences(&self) -> impl Iterator<Item = (Date, &Absence)> + '_ {
        self.absence
            .iter()
            .map(|absence| (self.make_date(absence.day()), absence))
    }

    pub fn holiday(&self) -> Option<&Holiday> {
        self.holiday.as_ref()
    }
}
