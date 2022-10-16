use core::fmt;

use thiserror::Error;

use crate::time::{Month, WeekDay, Year};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Date {
    year: Year,
    month: Month,
    day: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Error)]
#[error("{year:04}-{month:02}-{day:02}: not a valid date")]
pub struct InvalidDate {
    year: Year,
    month: Month,
    day: usize,
}

impl Date {
    pub fn new(year: impl Into<Year>, month: Month, day: usize) -> Result<Self, InvalidDate> {
        let year = year.into();
        if year.number_of_days_in_month(month) < day || day == 0 {
            return Err(InvalidDate { year, month, day });
        }

        Ok(Self { year, month, day })
    }

    pub fn week_day(&self) -> WeekDay {
        self.year().week_day(self.month(), self.day())
    }

    pub fn year(&self) -> Year {
        self.year
    }

    pub fn month(&self) -> Month {
        self.month
    }

    pub fn day(&self) -> usize {
        self.day
    }

    pub fn is_holiday(&self) -> bool {
        // check for christmas dates:
        self.month == Month::December && (self.day() == 25 || self.day() == 26) ||
        // new year's day
        self.month == Month::January && self.day() == 1 ||
        self.month == Month::January && self.day() == 6

        // TODO: add remaining holidays
        // https://github.com/kit-sdq/TimeSheetGenerator/blob/master/src/main/java/checker/holiday/GermanyHolidayChecker.java
        // https://www.dgb.de/gesetzliche-feiertage-deutschland-2020-2021#badenwuerttemberg
        // https://crates.io/crates/json_typegen/0.5.0
    }

    // TODO: might make this more powerful
    pub fn formatted(&self, f: &str) -> String {
        f.replace("{year}", &format!("{:04}", self.year()))
            .replace("{month}", &format!("{:02}", self.month()))
            .replace("{day}", &format!("{:02}", self.day()))
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.year.as_usize(),
            self.month.as_usize(),
            self.day
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_date() {
        //
        assert_eq!(
            Date::new(Year::new(2022), Month::January, 31).map(|d| d.to_string()),
            Ok("2022-01-31".to_string())
        );
    }
}
