use core::fmt;
use core::iter::Step;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Deserialize, Serialize)]
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
    pub fn months() -> impl IntoIterator<Item = Month> {
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

    pub fn as_usize(&self) -> usize {
        *self as usize
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
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
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
}
