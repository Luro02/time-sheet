use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::input::toml_input::{self, About, Contract, DynamicEntry, Entry, Mail, RepeatingEvent};
use crate::time::{Date, Month, Year};
use crate::utils::{self, ArrayVec};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    latex_mk_path: Option<PathBuf>,
    #[serde(default)]
    output_format: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Global {
    about: About,
    config: Option<Config>,
    mail: Option<Mail>,
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

    pub fn dynamic_repeating_in_month<'a>(
        &'a self,
        year: Year,
        month: Month,
        mut can_repeat_on: impl FnMut(Date) -> bool + 'a,
        department: &'a str,
    ) -> impl Iterator<Item = DynamicEntry> + 'a {
        // TODO: should be an associated function?
        self.repeating
            .iter()
            .filter_map(move |event| event.to_dynamic_entry(department).map(|e| (event, e)))
            .flat_map(move |(r, e)| {
                let mut entries: ArrayVec<_, 31> = ArrayVec::new();

                let range = Date::first_day(year, month)..=Date::last_day(year, month);
                for date in range.clone() {
                    if can_repeat_on(date) && r.repeats_on(date) {
                        entries.push(
                            e.clone()
                                .with_skip_dates(range.clone().filter(|d| d != &date).collect()),
                        );
                    }
                }

                entries
            })
    }

    #[must_use]
    pub fn resolve_output(&self, month: &toml_input::Month) -> String {
        let format = self
            .config
            .as_ref()
            .and_then(|c| c.output_format.as_ref())
            .map_or_else(
                || {
                    format!(
                        "{year:04}-{month:02}",
                        year = month.general().year(),
                        month = month.general().month()
                    )
                },
                |f| {
                    f.replace("{year}", &month.general().year().to_string())
                        .replace("{month}", &month.general().month().to_string())
                },
            );

        format!("{}.pdf", format)
    }

    #[must_use]
    pub fn mail(&self) -> Option<&Mail> {
        self.mail.as_ref()
    }
}
