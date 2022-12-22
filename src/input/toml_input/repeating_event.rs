use std::str::FromStr;

use serde::Deserialize;

use crate::input::toml_input::Entry;
use crate::time::{Date, TimeSpan, TimeStamp, WeekDay};
use crate::utils::{MapEntry, StrExt};

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

#[derive(Debug, Clone, PartialEq)]
pub enum CustomEnd {
    /// The event will never stop repeating.
    Never { start: Option<Date> },
    /// The date on which the event ends (inclusive).
    On { start: Option<Date>, end: Date },
    /// The event will stop repeating after `n` repetitions.
    AfterOccurrences { start: Date, count: usize },
}

impl CustomEnd {
    #[must_use]
    pub fn applies_on(&self, date: Date, previous_repetitions: impl FnOnce(Date) -> usize) -> bool {
        match self {
            Self::Never { start } => start.map_or(true, |start| start <= date),
            Self::On { start, end } => {
                if let Some(start) = start {
                    *start <= date && date <= *end
                } else {
                    date <= *end
                }
            }
            Self::AfterOccurrences { start, count } => {
                *start <= date && previous_repetitions(*start) < *count
            }
        }
    }
}

impl Default for CustomEnd {
    fn default() -> Self {
        Self::Never { start: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRepeatInterval {
    repeats_every: RepeatsEvery,
    repeats_on: [bool; 7],
    end: CustomEnd,
}

// when can the start date be omitted?
// - repeats_on weekdays
// - repeats every week or every day (but not monthly or yearly)

impl CustomRepeatInterval {
    pub fn new(repeats_every: RepeatsEvery, end: CustomEnd, repeats_on: Vec<WeekDay>) -> Self {
        // TODO: check if start date is required and correctly set repeats_on?

        Self {
            repeats_every,
            repeats_on: WeekDay::week_days().map(|day| repeats_on.contains(&day)),
            end,
        }
    }

    pub fn repeats_on(&self, date: Date) -> bool {
        self.repeats_on[date.week_day().as_usize() - 1]
            && self
                .end
                .applies_on(date, |start| self.repeats_every.repetitions(start, date))
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

// TODO: test that this works correctly, like one would expect
impl InternalRepeatingEvent {
    pub fn iter_week_days(&self) -> impl Iterator<Item = WeekDay> {
        match self {
            Self::WeekDays { repeats_on } => repeats_on.clone().into_iter(),
            Self::FixedStart { start_date } => vec![start_date.week_day()].into_iter(),
            Self::FixedDates { .. } => vec![].into_iter(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RepeatingEvent {
    #[serde(default)]
    action: String,
    repeats: RepeatSpan,
    #[serde(flatten)]
    internal: InternalRepeatingEvent,
    end_date: Option<Date>,
    start: TimeStamp,
    end: TimeStamp,
    #[serde(default)]
    department: Option<String>,
}

impl RepeatingEvent {
    pub const fn new_fixed_start(
        action: String,
        repeats: RepeatSpan,
        start: TimeStamp,
        end: TimeStamp,
        start_date: Date,
        end_date: Option<Date>,
        department: Option<String>,
    ) -> Self {
        Self {
            action,
            repeats,
            internal: InternalRepeatingEvent::FixedStart { start_date },
            start,
            end,
            end_date,
            department,
        }
    }

    pub const fn new_on_week_days(
        action: String,
        repeats: RepeatSpan,
        start: TimeStamp,
        end: TimeStamp,
        repeats_on: Vec<WeekDay>,
        end_date: Option<Date>,
        department: Option<String>,
    ) -> Self {
        Self {
            action,
            repeats,
            internal: InternalRepeatingEvent::WeekDays { repeats_on },
            start,
            end,
            end_date,
            department,
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

        self.to_custom().repeats_on(date)
    }

    #[must_use]
    fn to_custom(&self) -> CustomRepeatInterval {
        let start_date = {
            match &self.internal {
                InternalRepeatingEvent::WeekDays { .. } => None,
                InternalRepeatingEvent::FixedStart { start_date } => Some(*start_date),
                InternalRepeatingEvent::FixedDates { .. } => unimplemented!("not supported"),
            }
        };

        CustomRepeatInterval::new(
            RepeatsEvery::new(1, self.repeats),
            self.end_date.map_or_else(
                || CustomEnd::default(),
                |end| CustomEnd::On {
                    start: start_date,
                    end,
                },
            ),
            self.internal.iter_week_days().collect(),
        )
    }

    pub fn to_entry(&self, date: Date, department: &str) -> Option<Entry> {
        if !self.repeats_on(date) {
            return None;
        }

        // If a department is specified, only apply if the department matches
        if self.department.is_some() && self.department.as_deref() != Some(department) {
            return None;
        }

        // TODO: should `pause` be added?
        Some(Entry::new(
            date.day(),
            self.action.clone(),
            self.time_span(),
            None,
            None,
        ))
    }
}

impl<'de> MapEntry<'de> for RepeatingEvent {
    type Key = String;
    type Value = Self;

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.action = key;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::{date, time_stamp};

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct TomlParserDummy {
        #[serde(default, deserialize_with = "crate::utils::deserialize_map_entry")]
        repeating: Vec<RepeatingEvent>,
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
                repeating: vec![RepeatingEvent::new_on_week_days(
                    "regular catchup meeting".to_string(),
                    RepeatSpan::Week,
                    time_stamp!(09:15),
                    time_stamp!(11:00),
                    vec![WeekDay::Monday],
                    None,
                    None,
                )]
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
                repeating: vec![RepeatingEvent::new_fixed_start(
                    "regular catchup meeting".to_string(),
                    RepeatSpan::Month,
                    time_stamp!(12:35),
                    time_stamp!(15:21),
                    date!(2022:10:01),
                    Some(date!(2023:10:01)),
                    None,
                )]
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

    #[track_caller]
    fn assert_repeats_on(event: &RepeatingEvent, date: Date, expected: bool) {
        assert_eq!(
            event.repeats_on(date),
            expected,
            "event.repeats_on({}) should return \"{}\"",
            date,
            expected,
        );
    }

    #[test]
    fn test_repeats_on_weekdays() {
        let event = RepeatingEvent::new_on_week_days(
            "regular meeting".to_string(),
            RepeatSpan::Week,
            time_stamp!(08:00),
            time_stamp!(12:00),
            vec![WeekDay::Tuesday, WeekDay::Friday],
            None,
            None,
        );

        assert_eq!(
            event.to_custom(),
            CustomRepeatInterval::new(
                RepeatsEvery::new(1, RepeatSpan::Week),
                CustomEnd::Never { start: None },
                vec![WeekDay::Tuesday, WeekDay::Friday],
            )
        );

        for date in date!(2022:11:01)..=date!(2022:12:31) {
            assert_repeats_on(
                &event,
                date,
                date.week_day() == WeekDay::Tuesday || date.week_day() == WeekDay::Friday,
            );
        }
    }
}
