use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::input::toml_input::{About, Contract, Entry, RepeatingEvent};
use crate::time::{Date, Month, Year};
use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    latex_mk_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Global {
    about: About,
    config: Option<Config>,
    #[serde(deserialize_with = "utils::deserialize_map_entry")]
    contract: Vec<Contract>,
    #[serde(default, deserialize_with = "utils::deserialize_map_entry")]
    repeating: Vec<RepeatingEvent>,
}

impl Global {
    #[must_use]
    pub fn about(&self) -> &About {
        &self.about
    }

    #[must_use]
    pub fn contract(&self, department: &str) -> Option<&Contract> {
        self.contract
            .iter()
            .find(|contract| contract.department() == department)
    }

    #[must_use]
    pub fn latex_mk_path(&self) -> Option<&Path> {
        self.config
            .as_ref()
            .and_then(|config| config.latex_mk_path.as_deref())
    }

    pub fn repeating_in_month(&self, year: Year, month: Month) -> impl Iterator<Item = Entry> + '_ {
        (Date::first_day(year, month)..=Date::last_day(year, month))
            // check if it applies on that date and is not a holiday
            // TODO: should check for conflicts via month as well?
            .filter(|date| date.is_workday())
            .flat_map(|date| {
                self.repeating
                    .iter()
                    .filter_map(move |event| event.to_entry(date))
            })
    }
}
