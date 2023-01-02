use std::str::FromStr;

use anyhow::Context;
use serde::Deserialize;

use crate::input::toml_input::repeating::RepeatSpan;
use crate::time::Date;
use crate::utils::StrExt;

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub struct RepeatsEvery {
    n: usize,
    span: RepeatSpan,
}

impl RepeatsEvery {
    pub fn new(n: usize, span: RepeatSpan) -> Self {
        Self { n, span }
    }

    /// Returns how often an event has occured between `start` and `date`.
    ///
    /// If an event is on `date`, it is not counted.
    pub const fn repetitions(&self, start: Date, date: Date) -> usize {
        match self.span {
            RepeatSpan::Day => start.days_until(date) / self.n,
            RepeatSpan::Week => start.days_until(date) / (7 * self.n),
            RepeatSpan::Month => start.months_until(date) / self.n,
            RepeatSpan::Year => start.years_until(date) / self.n,
        }
    }
}

impl FromStr for RepeatsEvery {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let [Some(n), Some(span)] = s.split_exact::<2>(" ") {
            let n = n
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", s))?;
            let span = span.parse::<RepeatSpan>()?;

            Ok(Self::new(n, span))
        } else {
            let span = s
                .parse::<RepeatSpan>()
                .with_context(|| anyhow::anyhow!("Invalid repeats every: {}", s))?;

            Ok(Self::new(1, span))
        }
    }
}

impl TryFrom<String> for RepeatsEvery {
    type Error = <Self as FromStr>::Err;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::date;

    #[test]
    fn test_repetitions_days() {
        let mut elapsed_days = 0;

        for offset in 0..5 {
            assert_eq!(
                RepeatsEvery::new(5, RepeatSpan::Day)
                    .repetitions(date!(2022:12:04), date!(2022:12:04) + offset),
                0
            );
        }

        assert_eq!(
            RepeatsEvery::new(5, RepeatSpan::Day).repetitions(date!(2022:12:04), date!(2022:12:09)),
            1
        );

        assert_eq!(
            RepeatsEvery::new(1, RepeatSpan::Day).repetitions(date!(2022:12:04), date!(2022:12:09)),
            5
        );

        assert_eq!(
            RepeatsEvery::new(5, RepeatSpan::Day).repetitions(date!(2022:12:04), date!(2022:12:14)),
            2
        );

        let start = date!(2023:01:01);
        for date in start..=date!(2024:12:31) {
            for days in 1..=35 {
                assert_eq!(
                    RepeatsEvery::new(days, RepeatSpan::Day).repetitions(start, date),
                    elapsed_days / days
                );
            }

            elapsed_days += 1;
        }
    }

    #[test]
    fn test_repeats_every_week() {
        let repetition = RepeatsEvery::new(7, RepeatSpan::Day);

        for (passed_days, date) in (date!(2022:01:01)..=date!(2023:12:31)).enumerate() {
            assert_eq!(
                repetition.repetitions(date!(2022:01:01), date),
                passed_days / 7,
                "number of repetitions on day {} is not correct",
                date
            );
        }
    }
}
