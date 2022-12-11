use std::str::FromStr;

use serde::Deserialize;

use crate::date;
use crate::time::{Date, TimeSpan, TimeStamp, WeekDay};
use crate::utils::StrExt;

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(try_from = "String")]
#[serde(untagged)]
pub enum RepeatSpan {
    Day,
    Week,
    Month,
    Year,
}

impl FromStr for RepeatSpan {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "day" | "days" | "daily" => Ok(Self::Day),
            "week" | "weeks" | "weekly" => Ok(Self::Week),
            "month" | "months" | "monthly" => Ok(Self::Month),
            "year" | "years" | "yearly" => Ok(Self::Year),
            _ => Err(anyhow::anyhow!("Invalid repeat span: {}", s)),
        }
    }
}

impl TryFrom<String> for RepeatSpan {
    type Error = <Self as FromStr>::Err;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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

    #[must_use]
    pub fn applies_on(&self, start: Date, date: Date) -> bool {
        start == date || self.repetitions(start, date - 1) < self.repetitions(start, date)
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
            anyhow::bail!("Invalid repeats every: {}", s);
        }
    }
}

impl TryFrom<String> for RepeatsEvery {
    type Error = <Self as FromStr>::Err;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(try_from = "String")]
pub enum CustomEnd {
    /// The event will never stop repeating.
    #[default]
    Never,
    /// The date on which the event ends (inclusive).
    On(Date),
    /// The event will stop repeating after `n` repetitions.
    AfterOccurrences { count: usize },
}

impl CustomEnd {
    pub fn applies_on(&self, previous_repetitions: usize, date: Date) -> bool {
        match self {
            Self::Never => true,
            Self::On(end) => date <= *end,
            Self::AfterOccurrences { count } => previous_repetitions < *count,
        }
    }
}

impl From<Option<Date>> for CustomEnd {
    fn from(date: Option<Date>) -> Self {
        match date {
            Some(date) => Self::On(date),
            None => Self::Never,
        }
    }
}

impl FromStr for CustomEnd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "never" {
            return Ok(Self::Never);
        }

        if let [Some(count), Some("times")] = s.split_exact::<2>(" ") {
            let count = count
                .parse::<usize>()
                .map_err(|_| anyhow::anyhow!("Invalid number: {}", s))?;

            return Ok(Self::AfterOccurrences { count });
        }

        let date = s.parse::<Date>()?;
        Ok(Self::On(date))
    }
}

impl TryFrom<String> for CustomEnd {
    type Error = <Self as FromStr>::Err;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRepeatInterval {
    repeats_every: RepeatsEvery,
    repeats_on: [bool; 7],
    end: CustomEnd,
    start: Date,
}

impl CustomRepeatInterval {
    pub fn new(start: Date, repeats_every: RepeatsEvery, end: CustomEnd) -> Self {
        let mut result = Self {
            repeats_every,
            repeats_on: [false; 7],
            end,
            start,
        };

        result.set_repeats_on(start.week_day(), true);
        result
    }

    pub const fn repeats_on(&self, week_day: WeekDay) -> bool {
        self.repeats_on[week_day.as_usize() - 1]
    }

    pub fn set_repeats_on(&mut self, week_day: WeekDay, value: bool) -> &mut Self {
        self.repeats_on[week_day.as_usize() - 1] = value;
        self
    }

    pub fn applies_on(&self, date: Date) -> bool {
        let previous_repetitions = self.repeats_every.repetitions(self.start, date);

        self.repeats_on(date.week_day())
            && self.end.applies_on(previous_repetitions, date)
            && self.start <= date
            && self.repeats_every.applies_on(self.start, date)
    }
}

/*
How a repeat interval should look like:
[repeating."Regular catchup meeting"]
repeats_every = 2 weeks # 1 week, 1 month, ...
repeats_on = ["Thursday"]
ends = 10 times # never, 2022-01-01, ...

// alternatives for common repetitions:
[repeating."Regular catchup meeting"]
repeats = "weekly" # "daily", "monthly", "yearly"
*/

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum InternalRepeatingEvent {
    WeekDays { repeats_on: Vec<WeekDay> },
    FixedStart { start_date: Date },
    FixedDates { dates: Vec<Date> },
}

impl InternalRepeatingEvent {
    pub fn iter_week_days(&self) -> impl Iterator<Item = WeekDay> {
        match self {
            Self::WeekDays { repeats_on } => repeats_on.clone().into_iter(),
            Self::FixedStart { .. } => vec![].into_iter(),
            Self::FixedDates { .. } => vec![].into_iter(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RepeatingEvent {
    repeats: RepeatSpan,
    #[serde(flatten)]
    internal: InternalRepeatingEvent,
    end_date: Option<Date>,
    start: TimeStamp,
    end: TimeStamp,
}

impl RepeatingEvent {
    pub const fn new_fixed_start(
        repeats: RepeatSpan,
        start: TimeStamp,
        end: TimeStamp,
        start_date: Date,
        end_date: Option<Date>,
    ) -> Self {
        Self {
            repeats,
            internal: InternalRepeatingEvent::FixedStart { start_date },
            start,
            end,
            end_date,
        }
    }

    pub const fn new_on_week_days(
        repeats: RepeatSpan,
        start: TimeStamp,
        end: TimeStamp,
        repeats_on: Vec<WeekDay>,
        end_date: Option<Date>,
    ) -> Self {
        Self {
            repeats,
            internal: InternalRepeatingEvent::WeekDays { repeats_on },
            start,
            end,
            end_date,
        }
    }

    #[must_use]
    pub fn time_span(&self) -> TimeSpan {
        TimeSpan::new(self.start, self.end)
    }

    #[must_use]
    pub fn repeats_on(&self, date: Date) -> bool {
        if let InternalRepeatingEvent::FixedDates { dates } = &self.internal {
            return dates.contains(&date);
        }

        self.to_custom().applies_on(date)
    }

    #[must_use]
    fn to_custom(&self) -> CustomRepeatInterval {
        let start_date = {
            match &self.internal {
                InternalRepeatingEvent::WeekDays { repeats_on } => {
                    let first_week_day = *repeats_on.iter().min().unwrap();
                    let start_date = date!(1800:01:01).week_start();

                    start_date + (first_week_day.as_usize() - 1)
                }
                InternalRepeatingEvent::FixedStart { start_date } => *start_date,
                InternalRepeatingEvent::FixedDates { .. } => unimplemented!("not supported"),
            }
        };

        let mut interval = CustomRepeatInterval::new(
            start_date,
            RepeatsEvery::new(1, self.repeats),
            self.end_date.into(),
        );

        for week_day in self.internal.iter_week_days() {
            interval.set_repeats_on(week_day, true);
        }

        interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::{date, map, time_stamp};

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct TomlParserDummy {
        repeating: HashMap<String, RepeatingEvent>,
    }

    #[test]
    fn test_deserialize_shorthand() {
        assert_eq!(
            toml::from_str::<'_, TomlParserDummy>(concat!(
                "[repeating.\"regular catchup meeting\"]\n",
                "repeats = \"weekly\"\n",
                "start = \"09:15\"\n",
                "end = \"11:00\"\n",
                "repeats_on = [\"Monday\"]\n",
            )),
            Ok(TomlParserDummy {
                repeating: map! {
                    "regular catchup meeting".to_string() => RepeatingEvent::new_on_week_days(
                        RepeatSpan::Week,
                        time_stamp!(09:15),
                        time_stamp!(11:00),
                        vec![WeekDay::Monday],
                        None,
                    ),
                }
            })
        );

        assert_eq!(
            toml::from_str::<'_, TomlParserDummy>(concat!(
                "[repeating.\"regular catchup meeting\"]\n",
                "repeats = \"monthly\"\n",
                "start = \"12:35\"\n",
                "end = \"15:21\"\n",
                "start_date = \"2022-10-01\"\n",
                "end_date = \"2023-10-01\"\n",
            )),
            Ok(TomlParserDummy {
                repeating: map! {
                    "regular catchup meeting".to_string() => RepeatingEvent::new_fixed_start(
                        RepeatSpan::Month,
                        time_stamp!(12:35),
                        time_stamp!(15:21),
                        date!(2022:10:01),
                        Some(date!(2023:10:01)),
                    ),
                }
            })
        );
    }

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
    fn test_applies_on() {
        let repetition = RepeatsEvery::new(7, RepeatSpan::Day);

        for (passed_days, date) in (date!(2022:01:01)..=date!(2023:12:31)).enumerate() {
            assert_eq!(
                repetition.applies_on(date!(2022:01:01), date),
                passed_days % 7 == 0,
                "repetition on day {} is not correct",
                date
            );
        }
    }
}
