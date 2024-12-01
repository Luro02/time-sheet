use core::fmt;
use core::iter::Step;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(try_from = "usize")]
#[serde(into = "usize")]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    pub const fn new(number: usize) -> Self {
        Self::months()[number - 1]
    }

    pub const fn months() -> [Self; 12] {
        [
            Self::January,
            Self::February,
            Self::March,
            Self::April,
            Self::May,
            Self::June,
            Self::July,
            Self::August,
            Self::September,
            Self::October,
            Self::November,
            Self::December,
        ]
    }

    pub const fn as_usize(&self) -> usize {
        *self as usize
    }

    #[must_use]
    pub(crate) const fn is_eq(&self, other: &Self) -> bool {
        self.as_usize() == other.as_usize()
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        Self::months()[self.as_usize() % 12]
    }
}

impl From<Month> for usize {
    fn from(month: Month) -> Self {
        month.as_usize()
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_usize().fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Error)]
#[error("invalid month number")]
pub struct InvalidNumberForMonth;

impl TryFrom<usize> for Month {
    type Error = InvalidNumberForMonth;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::months()
            .into_iter()
            .find(|month| *month as usize == value)
            .ok_or(InvalidNumberForMonth)
    }
}

// TODO: test this?
impl Step for Month {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        <usize as Step>::steps_between(&start.as_usize(), &end.as_usize())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::forward_checked(start.as_usize(), count)
            .map(Self::try_from)?
            .ok()
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::backward_checked(start.as_usize(), count)
            .map(Self::try_from)?
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_display() {
        for month in Month::months() {
            assert_eq!(month.to_string(), month.as_usize().to_string());
        }
    }

    #[test]
    fn test_next() {
        assert_eq!(Month::December.next(), Month::January);
        assert_eq!(Month::January.next(), Month::February);
        assert_eq!(Month::February.next(), Month::March);
        assert_eq!(Month::March.next(), Month::April);
        assert_eq!(Month::April.next(), Month::May);
        assert_eq!(Month::May.next(), Month::June);
        assert_eq!(Month::June.next(), Month::July);

        let months = Month::months();
        for i in 0..months.len() {
            assert_eq!(months[i].next(), months[(i + 1) % months.len()]);
        }
    }
}
