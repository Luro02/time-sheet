use indexmap::IndexMap;
use serde::Deserialize;

use crate::input::toml_input::{DynamicEntry, Entry, General, Key, MultiEntry, Transfer};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EitherEntry {
    MultiEntry(MultiEntry),
    Entry(Entry),
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

#[derive(Debug, Clone, Deserialize)]
pub struct Month {
    general: General,
    transfer: Option<Transfer>,
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

    pub fn entries(&self) -> impl Iterator<Item = (&Key, &Entry)> {
        self.entries
            .iter()
            .flat_map(|(k, v)| v.into_iter().map(move |v| (k, v)))
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = (&String, &DynamicEntry)> + '_ {
        self.dynamic.iter()
    }
}
