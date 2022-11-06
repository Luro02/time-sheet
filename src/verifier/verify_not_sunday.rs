use thiserror::Error;

use crate::input::json_input::MonthFile;
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

    fn verify(&self, month_file: &MonthFile) -> Result<(), Self::Errors> {
        let errors = month_file
            .days()
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
