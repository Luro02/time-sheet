use std::collections::HashMap;

use serde::Deserialize;

use crate::input::toml_input::{About, Contract, EitherEntry, Entry, Key, RepeatingEvent};
use crate::time::{Date, Month, Year};

#[derive(Debug, Clone, Deserialize)]
pub struct Global {
    about: About,
    contract: HashMap<String, Contract>,
    repeating: HashMap<String, RepeatingEvent>,
}

impl Global {
    #[must_use]
    pub fn about(&self) -> &About {
        &self.about
    }

    #[must_use]
    pub fn contract(&self, department: &str) -> Option<&Contract> {
        self.contract.get(department)
    }

    pub fn repeating_in_month(
        &self,
        year: Year,
        month: Month,
    ) -> impl Iterator<Item = (Key, EitherEntry)> + '_ {
        (Date::first_day(year, month)..=Date::last_day(year, month)).filter_map(|date| {
            let mut events = Vec::new();
            for (name, event) in self.repeating.iter() {
                // check if it applies on that date and is not a holiday
                if event.repeats_on(date) && date.is_workday() {
                    events.push((name, event));
                }
            }

            if events.is_empty() {
                None
            } else {
                Some((
                    Key::from_day(date.day()),
                    EitherEntry::MultiEntry(
                        events
                            .into_iter()
                            .map(|(action, event)| {
                                // TODO: should `pause` be added?
                                Entry::new(
                                    action.to_string(),
                                    event.time_span().start(),
                                    event.time_span().end(),
                                    None,
                                    None,
                                )
                            })
                            .collect::<Vec<_>>()
                            .into(),
                    ),
                ))
            }
        })
    }
}
