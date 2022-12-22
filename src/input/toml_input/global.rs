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

    pub fn repeating_in_month<'a>(
        &'a self,
        year: Year,
        month: Month,
        mut can_repeat_on: impl FnMut(Date) -> bool + 'a,
        department: &'a str,
    ) -> impl Iterator<Item = Entry> + 'a {
        (Date::first_day(year, month)..=Date::last_day(year, month))
            // skip dates where the event cannot repeat
            .filter(move |date| can_repeat_on(*date))
            .flat_map(move |date| {
                self.repeating
                    .iter()
                    .filter_map(move |event| event.to_entry(date, department))
            })
    }
}
