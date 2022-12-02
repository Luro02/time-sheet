use std::iter::Step;
use std::ops::{Add, AddAssign, RangeInclusive};

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::time::{Date, Month, WeekDay};
use crate::utils::IteratorExt;
use crate::{iter_const, unreachable_unchecked};

#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize, Serialize, Display,
)]
#[serde(from = "usize")]
#[serde(into = "usize")]
#[display(fmt = "{}", _0)]
pub struct Year(usize);

/// The number of days from start_month..end_month in the `year`.
const fn days_for_months(year: Year, start_month: Month, end_month: usize) -> usize {
    /* TODO: replace with this, once the required features are const stabilized:
    let month_days = (month_ref..month)
        .into_iter()
        .map(|month| self.number_of_days_in_month(month))
        .sum::<usize>();
    */
    let mut result = 0;

    iter_const!(for month in start_month.as_usize(),..end_month => {
        result += year.number_of_days_in_month(Month::new(month));
    });

    result
}

impl Year {
    /// Choose the date 0000/01/01 as a base date, because it does not make sense to got past this date.
    const BASE_DATE: (Self, Month, usize, WeekDay) =
        (Self(0), Month::January, 1, WeekDay::Saturday);

    #[must_use]
    pub const fn new(year: usize) -> Self {
        Self(year)
    }

    #[must_use]
    pub const fn as_usize(&self) -> usize {
        self.0
    }

    /// A year that is not a leap year is a common year.
    pub const fn is_common_year(&self) -> bool {
        self.as_usize() % 4 != 0 || (self.as_usize() % 100 == 0 && self.as_usize() % 400 != 0)
    }

    /// A leap year is a calendar year that contains an additional day added to February, so
    /// it has 29 days instead of the regular 28 days.
    #[must_use]
    pub const fn is_leap_year(&self) -> bool {
        // https://en.wikipedia.org/wiki/Leap_year#Algorithm
        !self.is_common_year() && (self.as_usize() % 100 != 0 || self.as_usize() % 400 == 0)
    }

    #[must_use]
    pub const fn number_of_days_in_month(&self, month: Month) -> usize {
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
    #[must_use]
    pub const fn week_day(&self, month: Month, day: usize) -> WeekDay {
        let (year_ref, month_ref, day_ref, week_day_ref) = Self::BASE_DATE;

        // calculate the days elapsed between Self::BASE_DATE and self
        let days = {
            // something in here must be broken:
            let mut days = 0;

            // days between Month::January (= month_ref) and month
            days += days_for_months(*self, month_ref, month.as_usize());
            days += self.days_since(year_ref);
            days += day - day_ref;

            days
        };

        // this should be correct, because has been tested
        return week_day_ref.add_const(days);
    }

    /// Returns the number of days that have passed since `other`.
    ///
    /// `(other + self.days_since(other)) == self`
    // TODO: I think one could calculate this in O(1)?
    const fn days_since(&self, other: Self) -> usize {
        debug_assert!(self.as_usize() >= other.as_usize());

        let mut result = 0;
        iter_const!(for i in other.as_usize(),..self.as_usize() => {
            result += Year::new(i).days();
        });

        result
    }

    pub(super) const fn days_since_base_date(&self) -> usize {
        self.days_since(Self::BASE_DATE.0)
    }

    // TODO: improve algorithm?
    pub(super) const fn from_days_since_base_date(days: usize) -> Self {
        // Approximate the years upper/lower bounds:
        let lower_year = days / 366;
        let upper_year = days / 365;

        iter_const!(for year in lower_year,..upper_year + 1 => {
            let this_year = Year::new(year);
            let next_year = this_year.next();

            if this_year.days_since_base_date() <= days && next_year.days_since_base_date() > days {
                return this_year;
            }
        });

        unreachable_unchecked!("the year should always be found!")
    }

    /// Returns the number of days in this year.
    #[must_use]
    pub const fn days(&self) -> usize {
        if self.is_leap_year() {
            366
        } else {
            365
        }
    }

    /// Returns the number of weeks in this year's provided month.
    #[must_use]
    pub const fn number_of_weeks_in_month(&self, month: Month) -> usize {
        Date::last_day(*self, month).week_number()
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn iter_days_in(&self, month: Month) -> RangeInclusive<Date> {
        /*
        // for example 31 days
        let days = self.number_of_days_in_month(month);
        let year = *self;

        (1..=days)
            .into_iter()
            .map(move |day| Date::new(year, month, day).expect("date should be valid"))*/
        Date::first_day(*self, month)..=Date::last_day(*self, month)
    }

    #[must_use]
    pub fn iter_weeks_in(
        &self,
        month: Month,
    ) -> impl Iterator<Item = (usize, RangeInclusive<Date>)> + Clone {
        self.iter_days_in(month)
            .into_iter()
            .filter_map_with(0, |date, mut current_week| {
                let mut result = None;

                if current_week != date.week_number() {
                    current_week += 1;
                    result = Some((date.week_number(), date..=date.week_end()));
                }

                (result, current_week)
            })
    }
}

impl Add for Year {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.as_usize() + rhs.as_usize())
    }
}

impl Add<usize> for Year {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.as_usize() + rhs)
    }
}

impl AddAssign for Year {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<usize> for Year {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
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

impl From<Year> for usize {
    fn from(value: Year) -> Self {
        value.as_usize()
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
    fn test_days() {
        // this test runs under the assumption that year.is_leap_year works correctly
        for year in Year::new(1904)..=Year::new(3000) {
            if year.is_leap_year() {
                assert_eq!(year.days(), 366, "{} should have 366 days", year.as_usize());
            } else {
                assert_eq!(year.days(), 365, "{} should have 365 days", year.as_usize());
            }
        }
    }

    #[test]
    fn test_days_for_months() {
        let year = Year::new(2000);
        assert_eq!(
            days_for_months(year, Month::March, 13), // should be 306
            year.number_of_days_in_month(Month::March)
                + year.number_of_days_in_month(Month::April)
                + year.number_of_days_in_month(Month::May)
                + year.number_of_days_in_month(Month::June)
                + year.number_of_days_in_month(Month::July)
                + year.number_of_days_in_month(Month::August)
                + year.number_of_days_in_month(Month::September)
                + year.number_of_days_in_month(Month::October)
                + year.number_of_days_in_month(Month::November)
                + year.number_of_days_in_month(Month::December)
        );
    }

    #[test]
    fn test_days_since() {
        let base_year = Year::new(2000);

        let mut elapsed_days = 0;
        for year in base_year..=Year::new(2030) {
            assert_eq!(
                year.days_since(base_year),
                elapsed_days,
                "{} days since {}",
                year,
                base_year
            );
            elapsed_days += year.days();
        }
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

    #[test]
    fn test_from_days_since_base_date() {
        for year in Year::new(0)..=Year::new(3000) {
            let days_since_base_date = year.days_since_base_date();
            assert_eq!(
                Year::from_days_since_base_date(days_since_base_date),
                year,
                "{} days since base date should be {}",
                days_since_base_date,
                year
            );
        }
    }

    #[test]
    fn test_number_of_weeks() {
        fn test_assert_weeks_in_month(year: Year, month: Month, expected: usize) {
            assert_eq!(
                year.number_of_weeks_in_month(month),
                expected,
                "expected `{}` weeks in {:02}-{:04}",
                expected,
                month,
                year
            );
        }

        test_assert_weeks_in_month(Year::new(2022), Month::January, 6);
        test_assert_weeks_in_month(Year::new(2022), Month::February, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::March, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::April, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::May, 6);
        test_assert_weeks_in_month(Year::new(2022), Month::June, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::July, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::August, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::September, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::October, 6);
        test_assert_weeks_in_month(Year::new(2022), Month::November, 5);
        test_assert_weeks_in_month(Year::new(2022), Month::December, 5);
    }
}
