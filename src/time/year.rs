use std::iter::Step;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::time::{Month, WeekDay};

#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize, Serialize, Display,
)]
#[serde(from = "usize")]
#[serde(into = "usize")]
#[display(fmt = "{}", _0)]
pub struct Year(usize);

impl Year {
    /// Choose the date 0000/01/01 as a base date, because it does not make sense to got past this date.
    const BASE_DATE: (Self, Month, usize, WeekDay) =
        (Self(0), Month::January, 1, WeekDay::Saturday);

    #[must_use]
    pub fn new(year: usize) -> Self {
        Self(year)
    }

    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.0
    }

    /// A year that is not a leap year is a common year.
    pub fn is_common_year(&self) -> bool {
        self.as_usize() % 4 != 0 || (self.as_usize() % 100 == 0 && self.as_usize() % 400 != 0)
    }

    /// A leap year is a calendar year that contains an additional day added to February, so
    /// it has 29 days instead of the regular 28 days.
    #[must_use]
    pub fn is_leap_year(&self) -> bool {
        // https://en.wikipedia.org/wiki/Leap_year#Algorithm
        !self.is_common_year() && (self.as_usize() % 100 != 0 || self.as_usize() % 400 == 0)
    }

    #[must_use]
    pub fn number_of_days_in_month(&self, month: Month) -> usize {
        match month {
            Month::January => 31,
            Month::February => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            Month::March => 31,
            Month::April => 30,
            Month::May => 31,
            Month::June => 30,
            Month::July => 31,
            Month::August => 31,
            Month::September => 30,
            Month::October => 31,
            Month::November => 30,
            Month::December => 31,
        }
    }

    /// Calculate the weekday of this year and the specified month and day.
    ///
    /// # Note
    ///
    /// This function assumes that the day is valid.
    pub fn week_day(&self, month: Month, day: usize) -> WeekDay {
        let (year_ref, month_ref, day_ref, week_day_ref) = Self::BASE_DATE;

        let days = {
            let month_days = (month_ref..month)
                .into_iter()
                .map(|month| self.number_of_days_in_month(month))
                .sum::<usize>();

            let year_days = (year_ref..*self)
                .into_iter()
                .map(|year| year.days())
                .sum::<usize>();

            let day_days = day - day_ref;

            year_days + month_days + day_days
        };

        return week_day_ref + days;
    }

    pub fn days(&self) -> usize {
        Month::months()
            .into_iter()
            .map(|month| self.number_of_days_in_month(month))
            .sum()
    }
}

impl Step for Year {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        <usize as Step>::steps_between(&start.as_usize(), &end.as_usize())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::forward_checked(start.as_usize(), count).map(Year::new)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::backward_checked(start.as_usize(), count).map(Year::new)
    }
}

impl From<usize> for Year {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl Into<usize> for Year {
    fn into(self) -> usize {
        self.as_usize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_is_leap_year() {
        // from: https://www.calendar.best/leap-years.html
        macro_rules! assert_leap_years {
            ( $( $year:expr ),* $(,)? ) => {
                $(
                    assert!(
                        Year::new($year).is_leap_year(),
                        concat!(stringify!($year), " should be a leap year")
                    );
                )*
            };
        }

        macro_rules! assert_not_leap_years {
            ( $( $year:expr ),* $(,)? ) => {
                $(
                    assert!(
                        !Year::new($year).is_leap_year(),
                        concat!(stringify!($year), " should not be a leap year")
                    );
                )*
            };
        }

        assert_leap_years![
            1904, 1908, 1912, 1916, 1920, 1924, 1928, 1932, 1936, 1940, 1944, 1948, 1952, 1956,
            1960, 1964, 1968, 1972, 1976, 1980, 1984, 1988, 1992, 1996, 2000, 2004, 2008, 2012,
            2016, 2020, 2000, 2004, 2008, 2012, 2016, 2020, 2024, 2028, 2032, 2036, 2040, 2044,
            2048, 2052, 2056, 2060, 2064, 2068, 2072, 2076, 2080, 2084, 2088, 2092, 2096
        ];

        assert_not_leap_years![
            1900, 1901, 1902, 1903, 1905, 1906, 1907, 1909, 1910, 1911, 1913, 1914, 1915, 1917,
            1918, 1919, 1921, 1922, 1923, 1925, 1926, 1927, 1929, 1930, 1931, 2100, 2200, 2300,
            2500, 2600, 2700, 2900, 3000
        ];
    }

    #[test]
    fn test_week_day() {
        assert_eq!(Year::new(2000).week_day(Month::January, 2), WeekDay::Sunday);
        assert_eq!(Year::new(2000).week_day(Month::January, 3), WeekDay::Monday);
        assert_eq!(
            Year::new(2000).week_day(Month::January, 4),
            WeekDay::Tuesday
        );

        assert_eq!(
            Year::new(2001).week_day(Month::January, 15),
            WeekDay::Monday
        );
        assert_eq!(Year::new(2002).week_day(Month::March, 10), WeekDay::Sunday);
        assert_eq!(
            Year::new(2021).week_day(Month::December, 24),
            WeekDay::Friday
        );
    }

    #[test]
    fn test_year_range() {
        let mut iter = (Year::new(2000)..Year::new(2006)).into_iter();
        assert_eq!(iter.next(), Some(Year::new(2000)));
        assert_eq!(iter.next(), Some(Year::new(2001)));
        assert_eq!(iter.next(), Some(Year::new(2002)));
        assert_eq!(iter.next(), Some(Year::new(2003)));
        assert_eq!(iter.next(), Some(Year::new(2004)));
        assert_eq!(iter.next(), Some(Year::new(2005)));
        assert_eq!(iter.next(), None);

        let mut iter = (Year::new(2000)..Year::new(2000)).into_iter();
        assert_eq!(iter.next(), None);
    }
}
