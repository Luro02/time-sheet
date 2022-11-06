use std::collections::HashMap;

use serde::Deserialize;

use crate::input::toml_input::{Entry, General, Key, Transfer};

#[derive(Debug, Clone, Deserialize)]
pub struct Month {
    general: General,
    transfer: Option<Transfer>,
    entries: HashMap<Key, Entry>,
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
}
