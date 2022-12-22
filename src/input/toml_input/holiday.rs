use serde::Deserialize;

use crate::input::json_input::Entry;
use crate::input::toml_input::Task;
use crate::time::{Date, Month, TimeSpan, TimeStamp, WorkingDuration, Year};
use crate::working_duration;

const fn default_months() -> usize {
    1
}

const fn bool_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct Holiday {
    #[serde(default = "bool_true")]
    implicit: bool,
    #[serde(default)]
    start: Option<TimeStamp>,
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
        mut schedule: impl FnMut(Task) -> Vec<(Date, TimeSpan)>,
    ) -> Vec<Entry> {
        if !self.implicit {
            return vec![];
        }

        let duration = Self::duration(monthly_time, self.months);

        let date = Date::new(year, month, self.day).expect("invalid day for month");
        schedule({
            if let Some(start) = self.start {
                Task::new_with_start(duration, Some(date), true, start)
            } else {
                Task::new(duration, Some(date), true)
            }
        })
        .into_iter()
        .map(|(date, span)| Entry::new_vacation("Urlaub", date.day(), span.start(), span.end()))
        .collect()
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
