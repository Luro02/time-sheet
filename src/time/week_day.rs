use std::ops::Add;
use std::str::FromStr;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Deserialize)]
#[serde(try_from = "String")]
pub enum WeekDay {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl WeekDay {
    #[must_use]
    const fn week_days() -> [Self; 7] {
        [
            Self::Monday,
            Self::Tuesday,
            Self::Wednesday,
            Self::Thursday,
            Self::Friday,
            Self::Saturday,
            Self::Sunday,
        ]
    }

    /// The number of days that have to pass until `other` is reached.
    ///
    /// For example `Monday.days_until(Thursday)` would return `3`
    /// (`Monday`, `Tuesday`, `Wednesday`)
    #[must_use]
    pub const fn days_until(self, other: Self) -> usize {
        let start_index = self.as_usize();
        let end_index = other.as_usize();

        if start_index <= end_index {
            end_index - start_index
        } else {
            7 - (start_index - end_index)
        }
    }

    pub const fn as_usize(&self) -> usize {
        *self as usize
    }

    #[must_use]
    pub const fn add_const(self, days: usize) -> Self {
        Self::week_days()[(self.as_usize() - 1 + days % 7) % 7]
    }

    #[must_use]
    pub(crate) const fn is_eq(&self, other: &Self) -> bool {
        self.as_usize() == other.as_usize()
    }
}

impl Add<usize> for WeekDay {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        self.add_const(rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidWeekDayNumber;

impl TryFrom<usize> for WeekDay {
    type Error = InvalidWeekDayNumber;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::week_days()
            .into_iter()
            .find(|v| *v as usize == value)
            .ok_or(InvalidWeekDayNumber)
    }
}

impl FromStr for WeekDay {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_lowercase();

        match value.as_str() {
            "monday" => Ok(Self::Monday),
            "tuesday" => Ok(Self::Tuesday),
            "wednesday" => Ok(Self::Wednesday),
            "thursday" => Ok(Self::Thursday),
            "friday" => Ok(Self::Friday),
            "saturday" => Ok(Self::Saturday),
            "sunday" => Ok(Self::Sunday),
            _ => Err(anyhow::anyhow!("Invalid week day: {}", value)),
        }
    }
}

impl TryFrom<String> for WeekDay {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_days_until() {
        for (distance, week_day) in WeekDay::week_days().into_iter().enumerate() {
            assert_eq!(WeekDay::Monday.days_until(week_day), distance);
        }

        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Monday), 6);
        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Tuesday), 0);
        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Wednesday), 1);
        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Thursday), 2);
        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Friday), 3);
        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Saturday), 4);
        assert_eq!(WeekDay::Tuesday.days_until(WeekDay::Sunday), 5);

        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Monday), 1);
        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Tuesday), 2);
        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Wednesday), 3);
        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Thursday), 4);
        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Friday), 5);
        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Saturday), 6);
        assert_eq!(WeekDay::Sunday.days_until(WeekDay::Sunday), 0);

        assert_eq!(WeekDay::Wednesday.days_until(WeekDay::Sunday), 4);

        assert_eq!(WeekDay::Thursday.days_until(WeekDay::Thursday), 0);
        assert_eq!(WeekDay::Thursday.days_until(WeekDay::Friday), 1);
        assert_eq!(WeekDay::Thursday.days_until(WeekDay::Saturday), 2);
        assert_eq!(WeekDay::Thursday.days_until(WeekDay::Sunday), 3);
        assert_eq!(WeekDay::Thursday.days_until(WeekDay::Monday), 4);
    }

    #[test]
    fn test_add() {
        assert_eq!(WeekDay::Monday + 1, WeekDay::Tuesday);
        assert_eq!(WeekDay::Tuesday + 1, WeekDay::Wednesday);
        assert_eq!(WeekDay::Wednesday + 1, WeekDay::Thursday);
        assert_eq!(WeekDay::Thursday + 1, WeekDay::Friday);
        assert_eq!(WeekDay::Friday + 1, WeekDay::Saturday);
        assert_eq!(WeekDay::Saturday + 1, WeekDay::Sunday);
        assert_eq!(WeekDay::Sunday + 1, WeekDay::Monday);

        for week_day in WeekDay::week_days() {
            assert_eq!(week_day + 0, week_day);
            assert_eq!(week_day + 2, (week_day + 1) + 1);
            assert_eq!(week_day + 3, (week_day + 2) + 1);
            assert_eq!(week_day + 4, (week_day + 3) + 1);
            assert_eq!(week_day + 5, (week_day + 4) + 1);
            assert_eq!(week_day + 6, (week_day + 5) + 1);
        }
    }

    #[test]
    fn test_add_overflow() {
        for week_day in WeekDay::week_days() {
            for i in 0..=365 {
                assert_eq!(week_day + i, week_day + (i % 7));
            }
        }
    }
}
