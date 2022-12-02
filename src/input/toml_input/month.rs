use indexmap::IndexMap;
use serde::Deserialize;

use crate::input::toml_input::{DynamicEntry, Entry, General, Holiday, Key, MultiEntry, Transfer};
use crate::time::WorkingDuration;

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
    dynamic: IndexMap<String, DynamicEntry>,
}

impl Month {
    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn transfer(&self) -> Option<&Transfer> {
        self.transfer.as_ref()
    }

    pub fn entries(&self, monthly_time: WorkingDuration) -> impl Iterator<Item = (Key, Entry)> {
        let iter = self
            .holiday
            .as_ref()
            .and_then(|h| h.to_entry(self.general.year(), self.general.month(), monthly_time))
            .into_iter();

        self.entries
            .clone()
            .into_iter()
            .flat_map(|(k, v)| v.into_iter().map(move |v| (k.clone(), v)))
            .chain(iter)
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = (&String, &DynamicEntry)> + '_ {
        self.dynamic.iter()
    }
}
