use std::collections::HashMap;

use serde::Deserialize;

use crate::input::toml_input::{DynamicEntry, Entry, General, Key, Transfer};

#[derive(Debug, Clone, Deserialize)]
pub struct Month {
    general: General,
    transfer: Option<Transfer>,
    entries: HashMap<Key, Entry>,
    dynamic: HashMap<String, DynamicEntry>,
}

impl Month {
    pub fn general(&self) -> &General {
        &self.general
    }

    pub fn transfer(&self) -> Option<&Transfer> {
        self.transfer.as_ref()
    }

    pub fn entries(&self) -> impl Iterator<Item = (&Key, &Entry)> {
        self.entries.iter()
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = (&String, &DynamicEntry)> + '_ {
        self.dynamic.iter()
    }
}
