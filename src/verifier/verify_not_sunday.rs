use thiserror::Error;

use crate::input::Config;
use crate::time::{Date, WeekDay};
use crate::verifier::Verifier;

pub struct VerifyNotSunday;

#[derive(Debug, Clone, Error, PartialEq)]
#[error("{date}: you are not supposed to work on sundays")]
pub struct SundayNotAllowed {
    date: Date,
}

impl Verifier for VerifyNotSunday {
    type Error = SundayNotAllowed;
    type Errors = Vec<SundayNotAllowed>;

    fn verify(&self, config: &Config) -> Result<(), Self::Errors> {
        let month_config = config.month();
        let month = month_config.month();
        let year = month_config.year();

        let errors = year
            .iter_days_in(month)
            .filter(|date| month_config.has_entries_on(*date))
            .filter_map(|date| {
                (date.week_day() == WeekDay::Sunday).then(|| SundayNotAllowed { date })
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}
