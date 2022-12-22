use serde::de;
use serde::Deserialize;

use crate::time::{Date, TimeSpan, TimeStamp, WorkingDuration};
use crate::time_stamp;
use crate::utils::{MapEntry, StrExt};

#[derive(Debug, Clone, PartialEq)]
pub enum AbsenceKey {
    Day(usize),
    Range { start: usize, end: usize },
}

impl Default for AbsenceKey {
    fn default() -> Self {
        Self::Day(0)
    }
}

fn parse_day_or_error<'de, D>(input: &str) -> Result<usize, D::Error>
where
    D: de::Deserializer<'de>,
{
    let number = input.parse::<usize>().map_err(de::Error::custom)?;

    if number == 0 || number > 31 {
        return Err(de::Error::custom(format!(
            "day must be between 1 and 31, but was {}",
            number
        )));
    }

    Ok(number)
}

impl<'de> de::Deserialize<'de> for AbsenceKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;

        if let [Some(start_str), Some(end_str)] = input.split_exact::<2>("-") {
            let start = parse_day_or_error::<D>(start_str)?;
            let end = parse_day_or_error::<D>(end_str)?;

            return Ok(Self::Range { start, end });
        }

        let number = parse_day_or_error::<D>(&input)?;

        Ok(Self::Day(number))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Absence {
    #[serde(default)]
    key: AbsenceKey,
    /// When the absence starts on the first day.
    start: TimeStamp,
    /// When the absence ends on the last day.
    end: TimeStamp,
}

impl Absence {
    #[must_use]
    const fn first_day(&self) -> usize {
        match self.key {
            AbsenceKey::Day(day) => day,
            AbsenceKey::Range { start, .. } => start,
        }
    }

    #[must_use]
    const fn last_day(&self) -> usize {
        match self.key {
            AbsenceKey::Day(day) => day,
            AbsenceKey::Range { end, .. } => end,
        }
    }

    #[must_use]
    pub const fn time_span(&self) -> TimeSpan {
        TimeSpan::new(self.start, self.end)
    }

    pub const fn duration(&self) -> WorkingDuration {
        self.time_span().duration()
    }

    pub fn to_date_absences<'a>(
        &'a self,
        make_date: impl Fn(usize) -> Date + 'a,
    ) -> impl Iterator<Item = (Date, Self)> + 'a {
        let first_day = self.first_day();
        let last_day = self.last_day();

        (first_day..=last_day).map(move |day| {
            let mut start = time_stamp!(00:00);
            let mut end = time_stamp!(23:59);
            if day == first_day {
                start = self.start;
            }

            if day == last_day {
                end = self.end;
            }

            (
                make_date(day),
                Self {
                    key: AbsenceKey::Day(day),
                    start,
                    end,
                },
            )
        })
    }
}

impl<'de> MapEntry<'de> for Absence {
    type Key = AbsenceKey;
    type Value = Self;

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.key = key;
        value
    }
}
