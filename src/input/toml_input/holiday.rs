use serde::Deserialize;

use crate::input::toml_input::{Entry, Key};
use crate::time::{Date, Month, WorkingDuration, Year};
use crate::{time_stamp, working_duration};

const fn default_months() -> usize {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct Holiday {
    implicit: bool,
    day: usize,
    #[serde(default = "default_months")]
    months: usize,
}

const fn divide_and_round(dividend: usize, divisor: usize) -> usize {
    (dividend + (divisor / 2)) / divisor
}

impl Holiday {
    const MAXIMUM_HOURS_PER_MONTH: WorkingDuration = working_duration!(85:00);
    const HOLIDAYS_PER_YEAR: usize = 20;
    const SCALAR_IN_MINS: usize = 237; // 3.95h * 60min

    #[must_use]
    const fn duration(monthly_time: WorkingDuration, months: usize) -> WorkingDuration {
        let mins_per_month = monthly_time.as_mins() as usize;

        let dividend = mins_per_month * Self::HOLIDAYS_PER_YEAR * Self::SCALAR_IN_MINS * months;
        let divisor = Self::MAXIMUM_HOURS_PER_MONTH.as_mins() as usize * 12;

        WorkingDuration::from_mins(divide_and_round(dividend, divisor) as u16)
    }

    #[must_use]
    pub fn is_implicit(&self) -> bool {
        self.implicit
    }

    #[must_use]
    pub fn to_entry(
        &self,
        year: Year,
        month: Month,
        monthly_time: WorkingDuration,
    ) -> Option<(Key, Entry)> {
        if !self.implicit {
            return None;
        }

        let duration = Self::duration(monthly_time, self.months);

        let start = time_stamp!(11:00);
        let end = start + duration;

        let mut date = Date::new(year, month, self.day).unwrap();
        while !date.is_workday() {
            date += 1;

            if date.month() != month {
                panic!("Could not find a workday for the holiday")
            }
        }

        Some((
            Key::from_day(date.day()),
            Entry::new("Urlaub".to_string(), start, end, None, Some(true)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_calculate_duration() {
        assert_eq!(
            Holiday::duration(working_duration!(40:00), 5),
            working_duration!(15:29)
        );
    }

    #[test]
    fn test_divide_and_round() {
        for left in 0..=1_000 {
            for right in 1..=1_000 {
                let expected = ((left as f64) / (right as f64)).round() as usize;
                assert_eq!(
                    divide_and_round(left, right),
                    expected,
                    "{} / {} should be {}",
                    left,
                    right,
                    expected
                );
            }
        }
    }
}
