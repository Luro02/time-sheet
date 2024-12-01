use core::fmt;
use core::iter::Step;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

use crate::time::{holiday, Month, WeekDay, Year};
use crate::utils::StrExt;

#[macro_export]
macro_rules! date {
    ($year:literal : $month:literal : $day:literal) => {{
        const _YEAR: $crate::time::Year = $crate::time::Year::new($year);
        static_assertions::const_assert!($month >= 1 && $month <= 12);

        const _MONTH: $crate::time::Month = $crate::time::Month::new($month);

        // validate the day
        static_assertions::const_assert!($day != 0);
        static_assertions::const_assert!($day <= _YEAR.number_of_days_in_month(_MONTH));

        unsafe { $crate::time::Date::new_unchecked(_YEAR, _MONTH, $day) }
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct Date {
    year: Year,
    month: Month,
    day: usize,
}

impl Date {
    pub fn new(year: impl Into<Year>, month: Month, day: usize) -> Result<Self, InvalidDate> {
        let year = year.into();
        if year.number_of_days_in_month(month) < day || day == 0 {
            return Err(InvalidDate::InvalidDay { year, month, day });
        }

        Ok(Self { year, month, day })
    }

    #[doc(hidden)]
    #[must_use]
    pub const unsafe fn new_unchecked(year: Year, month: Month, day: usize) -> Self {
        Self { year, month, day }
    }

    /// Returns the date of the first day as a date in the month.
    #[must_use]
    pub const fn first_day(year: Year, month: Month) -> Self {
        Self {
            year,
            month,
            day: 1,
        }
    }

    /// Returns the date of the last day as a date in the month.
    #[must_use]
    pub const fn last_day(year: Year, month: Month) -> Self {
        Self {
            year,
            month,
            day: year.number_of_days_in_month(month),
        }
    }

    #[must_use]
    const fn from_ordinal(year: Year, ordinal: u16) -> Self {
        if year.days() < ordinal as usize || ordinal == 0 {
            const_panic::concat_panic!(
                "Invalid ordinal `",
                ordinal,
                "` for year ",
                year.as_usize(),
                " with ",
                year.days(),
                " days."
            );
        }

        let cumulative_days = year.cumulative_days();

        // this is in O(1) as the number of months is bounded by 12
        // under the assumption that the compiler is smart enough to
        // understand that .next() is not causing an infinite loop
        let mut current_month = Month::January;
        while !current_month.is_eq(&Month::December)
            && cumulative_days[current_month.as_usize()] < ordinal as usize
        {
            current_month = current_month.next();
        }

        let day = ordinal as usize - cumulative_days[current_month.as_usize() - 1];

        Self {
            year,
            month: current_month,
            day,
        }
    }

    #[must_use]
    const fn from_days_since_base_date(days: usize) -> Self {
        let year = Year::from_days_since_base_date(days);
        // NOTE: +1 because the ordinal of the first day of the year is 1 and not 0
        let ordinal = (days - year.days_since_base_date()) + 1;
        Self::from_ordinal(year, ordinal as u16)
    }
}

impl Date {
    // TODO: might make this more powerful
    pub fn formatted(&self, f: &str) -> String {
        f.replace("{year}", &format!("{:04}", self.year()))
            .replace("{month}", &format!("{:02}", self.month()))
            .replace("{day}", &format!("{:02}", self.day()))
    }
}

impl Date {
    pub const fn week_day(&self) -> WeekDay {
        self.year().week_day(self.month(), self.day())
    }

    pub const fn year(&self) -> Year {
        self.year
    }

    pub const fn month(&self) -> Month {
        self.month
    }

    pub const fn day(&self) -> usize {
        self.day
    }

    // TODO: write some good tests for this, also take care of https://github.com/kit-sdq/TimeSheetGenerator/pull/121
    pub const fn is_holiday(&self) -> bool {
        holiday::is_holiday(*self)
    }

    #[must_use]
    const fn apply_offset(week_day: WeekDay, day: usize) -> usize {
        let offset = week_day as usize - 1;

        // In rust divisions always round down.
        // Dividing any number x by 7 for which holds:
        // 7 * n <= x < 7 * (n + 1) will result in n
        //
        // The first week number is 1 and not 0, so to each day 7 is added.
        //
        // Then the offset is added to the day, so that all mondays are a multiple of 7.
        // (one can calculate the week_numbers for weeks starting not on monday the same
        //  way, just make the day where the week starts a multiple of 7)
        //
        // Months starting with a monday will have the days 1, 8, 15, 22, 29
        // The offset is added so that they will be 0, 7, 14, 21, 28 (or with the + 7):
        // 7, 14, 21, 28, 35
        //  7 / 7 = 1
        // 14 / 7 = 2
        // 21 / 7 = 3
        // 28 / 7 = 4
        // 35 / 7 = 5
        day + 7 + offset - 1
    }

    #[must_use]
    pub const fn week_number(&self) -> usize {
        Self::apply_offset(
            Self::first_day(self.year(), self.month()).week_day(),
            self.day(),
        ) / 7
    }

    #[must_use]
    pub const fn week_start(&self) -> Self {
        Self {
            year: self.year(),
            month: self.month(),
            day: {
                let distance = WeekDay::Monday.days_until(self.week_day());
                if self.day() <= distance {
                    1
                } else {
                    self.day() - distance
                }
            },
        }
    }

    /// Returns the date of the last day in the current week.
    #[must_use]
    pub const fn week_end(&self) -> Self {
        Self {
            year: self.year(),
            month: self.month(),
            day: {
                let distance = self.week_day().days_until(WeekDay::Sunday);
                if self.day() + distance > self.year().number_of_days_in_month(self.month()) {
                    self.year().number_of_days_in_month(self.month())
                } else {
                    self.day() + distance
                }
            },
        }
    }

    #[must_use]
    pub const fn is_workday(&self) -> bool {
        !self.is_holiday() && !self.week_day().is_eq(&WeekDay::Sunday)
    }

    #[must_use]
    const fn ordinal(&self) -> u16 {
        let mut result = 0;

        // -1 to get the index of the previous month
        // will not cause a panic, because the first month
        // (january) has the number 1
        result += self.year().cumulative_days()[self.month().as_usize() - 1] as u16;

        result + self.day() as u16
    }

    #[must_use]
    const fn days_since_base_date(&self) -> usize {
        // the ordinal of the first day of the year is 1.
        // when one does not subtract 1, then
        // date!(0000:01:01).days_since_base_date()
        // = 0 + 1 (because ordinal is 1)
        //
        // but this is not correct => one has to subtract 1
        self.year.days_since_base_date() + (self.ordinal() - 1) as usize
    }

    #[must_use]
    pub(super) const fn add_days(self, days: usize) -> Self {
        let mut ordinal = self.ordinal() as usize + days;
        let mut year = self.year();

        // TODO: could this be calculated in O(1)?
        while ordinal > year.days() {
            ordinal -= year.days();
            year = year.next();
        }

        Self::from_ordinal(year, ordinal as u16)
    }

    #[must_use]
    pub(super) const fn sub_days(self, days: usize) -> Self {
        let mut ordinal = self.ordinal() as usize;
        let mut year = self.year();

        while ordinal < days {
            year = year.prev();
            ordinal += year.days();
        }

        if ordinal == days {
            year = year.prev();
            ordinal = year.days();
        } else {
            ordinal -= days;
        }

        Self::from_ordinal(year, ordinal as u16)
    }

    /// Returns the date when the next week starts or `None` if the next week
    /// would be in the next month.
    #[must_use]
    pub const fn next_week_start(&self) -> Option<Self> {
        let next_week = self.week_start().add_days(7);

        if next_week.month().is_eq(&self.month()) {
            Some(next_week.week_start())
        } else {
            None
        }
    }

    /// Returns the number of days that have passed between `self` and `other`.
    ///
    /// `self + self.days_until(other) == other`
    ///
    /// # Panics
    ///
    /// This function assumes that `self` is before `other`.
    /// If this is not the case, it will panic.
    #[must_use]
    pub const fn days_until(&self, other: Self) -> usize {
        other.days_since_base_date() - self.days_since_base_date()
    }

    /// Returns the number of years that have passed between `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use time_sheet::time::Date;
    /// # use time_sheet::date;
    /// assert_eq!(
    ///     date!(2022:01:01).years_until(date!(2023:01:01)),
    ///     1
    /// );
    /// ```
    ///
    /// # Panics
    ///
    /// This function assumes that `self` is before `other`.
    /// If this is not the case, it will panic.
    #[must_use]
    pub const fn years_until(&self, other: Self) -> usize {
        let mut years = other.year().as_usize() - self.year().as_usize();

        if self.month().as_usize() > other.month().as_usize()
            || (self.month().is_eq(&other.month()) && self.day() > other.day())
        {
            years -= 1;
        }

        years
    }

    /// Returns the number of months that have passed between `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use time_sheet::time::Date;
    /// # use time_sheet::date;
    /// assert_eq!(
    ///     date!(2022:01:01).months_until(date!(2022:02:01)),
    ///     1
    /// );
    /// ```
    ///
    /// # Panics
    ///
    /// This function assumes that `self` is before `other`.
    /// If this is not the case, it will panic.
    #[must_use]
    pub const fn months_until(&self, other: Self) -> usize {
        let previous_months = self.years_until(other) * 12;

        if self.day() <= other.day() {
            previous_months + other.month().as_usize() - self.month().as_usize()
        } else {
            previous_months + other.month().as_usize() - self.month().as_usize() - 1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidDate {
    #[error("\"{input}\" is not valid date. Expected format: \"YYYY-MM-DD\"")]
    ParseDateError { input: String },
    #[error("{day:02} is not a valid day for {year:04}-{month:02}")]
    InvalidDay {
        year: Year,
        month: Month,
        day: usize,
    },
}

impl Add<usize> for Date {
    type Output = Self;

    fn add(self, days: usize) -> Self::Output {
        self.add_days(days)
    }
}

impl Sub<usize> for Date {
    type Output = Self;

    fn sub(self, days: usize) -> Self::Output {
        self.sub_days(days)
    }
}

impl SubAssign<usize> for Date {
    fn sub_assign(&mut self, days: usize) {
        *self = *self - days;
    }
}

impl AddAssign<usize> for Date {
    fn add_assign(&mut self, days: usize) {
        *self = self.add_days(days);
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.year.as_usize(),
            self.month.as_usize(),
            self.day
        )
    }
}

impl Step for Date {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        <usize as Step>::steps_between(&start.days_since_base_date(), &end.days_since_base_date())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::forward_checked(start.days_since_base_date(), count)
            .map(Self::from_days_since_base_date)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        <usize as Step>::backward_checked(start.days_since_base_date(), count)
            .map(Self::from_days_since_base_date)
    }
}

fn parse_or_err(input: &str) -> Result<usize, InvalidDate> {
    input
        .parse::<usize>()
        .map_err(|_| InvalidDate::ParseDateError {
            input: input.to_string(),
        })
}

impl FromStr for Date {
    type Err = InvalidDate;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let [Some(year), Some(month), Some(day)] = string.split_exact::<3>("-") {
            let year = Year::new(parse_or_err(year)?);
            let month =
                Month::try_from(parse_or_err(month)?).map_err(|_| InvalidDate::ParseDateError {
                    input: string.to_string(),
                })?;
            let day = parse_or_err(day)?;

            Self::new(year, month, day)
        } else {
            Err(InvalidDate::ParseDateError {
                input: string.to_string(),
            })
        }
    }
}

impl TryFrom<String> for Date {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<Date> for toml::value::Date {
    fn from(date: Date) -> Self {
        toml::value::Date {
            year: date.year().as_usize() as u16,
            month: date.month() as u8,
            day: date.day() as u8,
        }
    }
}

impl TryFrom<toml::value::Date> for Date {
    type Error = InvalidDate;

    fn try_from(date: toml::value::Date) -> Result<Self, Self::Error> {
        Self::new(
            Year::new(date.year as usize),
            Month::try_from(date.month as usize).unwrap(),
            date.day as usize,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::min;
    use crate::utils::IteratorExt;
    use std::ops::RangeInclusive;

    #[test]
    fn test_date_to_string() {
        //
        assert_eq!(
            Date::new(Year::new(2022), Month::January, 31).map(|d| d.to_string()),
            Ok("2022-01-31".to_string())
        );
    }

    #[must_use]
    fn sort_array<T: Ord, const N: usize>(mut array: [T; N]) -> [T; N] {
        array.sort();
        array
    }

    #[test]
    fn test_date_sorting() {
        assert_eq!(
            sort_array([date!(2022:01:03), date!(2022:01:02), date!(2022:01:01)]),
            [date!(2022:01:01), date!(2022:01:02), date!(2022:01:03)]
        );

        assert_eq!(
            sort_array([date!(2012:01:03), date!(2013:01:02), date!(2024:01:01)]),
            [date!(2012:01:03), date!(2013:01:02), date!(2024:01:01)]
        );

        assert_eq!(
            sort_array([date!(2000:01:01), date!(2000:04:01), date!(2000:03:01)]),
            [date!(2000:01:01), date!(2000:03:01), date!(2000:04:01)]
        );
    }

    #[test]
    fn test_add_day() {
        assert_eq!(date!(2022:01:01).add_days(1), date!(2022:01:02));
        assert_eq!(date!(2022:01:01).add_days(30), date!(2022:01:31));
        assert_eq!(date!(2022:01:01).add_days(31), date!(2022:02:01));
        assert_eq!(date!(2022:01:01).add_days(58), date!(2022:02:28));
        assert_eq!(date!(2022:01:01).add_days(59), date!(2022:03:01));

        assert_eq!(date!(2022:12:24).add_days(8), date!(2023:01:01));
        assert_eq!(date!(2022:12:24).add_days(8 + 365), date!(2024:01:01));
    }

    #[test]
    fn test_sub_days() {
        assert_eq!(
            Date::from_days_since_base_date(date!(2022:12:31).days_since_base_date() + 1),
            date!(2023:01:01)
        );
        assert_eq!(
            Date::from_days_since_base_date(date!(2022:12:31).days_since_base_date() + 2),
            date!(2023:01:02)
        );

        assert_eq!(date!(2022:01:01).sub_days(0), date!(2022:01:01));

        assert_eq!(date!(2024:01:01).sub_days(0), date!(2024:01:01));
        assert_eq!(date!(2024:01:01).sub_days(1), date!(2023:12:31));
        assert_eq!(date!(2024:01:01).sub_days(2), date!(2023:12:30));
        assert_eq!(date!(2024:01:01).sub_days(364), date!(2023:01:02));
        assert_eq!(date!(2024:01:01).sub_days(365), date!(2023:01:01));
        assert_eq!(date!(2024:01:01).sub_days(729), date!(2022:01:02));
        assert_eq!(date!(2024:01:01).sub_days(730), date!(2022:01:01));

        let start = date!(2020:01:01);
        for (passed_days, date) in (start..=date!(2024:12:31)).enumerate() {
            assert_eq!(
                date.sub_days(passed_days),
                start,
                "expected `{}` - `{}` = `{}`, but got `{}`",
                date,
                passed_days,
                start,
                date.sub_days(passed_days)
            );
        }
    }

    // TODO: write tests that
    // - (a + b) - b = a
    // - (a - b) + b = a
    // - a + (b - c) = (a + b) - c
    // - a - (b - c) = (a - b) + c

    #[test]
    fn test_add_sub_identity() {
        for a in date!(2022:01:01)..=date!(2024:12:31) {
            for b in 0..=999 {
                assert_eq!(a.add_days(b).sub_days(b), a);
                assert_eq!(a.sub_days(b).add_days(b), a);
            }
        }
    }

    #[test]
    fn test_ordinal() {
        assert_eq!(date!(2022:01:01).ordinal(), 1);
        assert_eq!(date!(2022:02:01).ordinal(), 32);
        assert_eq!(date!(2022:02:05).ordinal(), 36);

        for year in Year::new(2020)..=Year::new(3000) {
            let mut current_ordinal = 0;
            for month in Month::months() {
                for day in 1..=year.number_of_days_in_month(month) {
                    current_ordinal += 1;
                    let date = Date::new(year, month, day).unwrap();

                    assert_eq!(date.ordinal(), current_ordinal);
                }
            }
        }
    }

    #[test]
    fn test_from_days_since_base_date() {
        for year in Year::new(2020)..=Year::new(2025) {
            for month in Month::months() {
                for day in 1..year.number_of_days_in_month(month) {
                    let date = Date::new(year, month, day).unwrap();

                    assert_eq!(
                        Date::from_days_since_base_date(date.days_since_base_date()),
                        date
                    );
                }
            }
        }
    }

    #[inline]
    #[track_caller]
    fn test_week_number_value(
        year: Year,
        month: Month,
        expected: usize,
        days: impl IntoIterator<Item = usize>,
    ) {
        for day in days {
            let actual = Date::new(year, month, day).unwrap().week_number();
            assert_eq!(
                expected, actual,
                "week_number({}-{}-{:02}): expected: {}, actual: {}",
                year, month, day, expected, actual,
            );
        }
    }

    fn iter_weeks(year: Year, month: Month) -> Vec<(usize, RangeInclusive<usize>)> {
        let mut result = Vec::new();
        // NOTE: if monday is the first day, then this will be 0
        let day_before_first_monday = Date::new(year, month, 1)
            .unwrap()
            .week_day()
            .days_until(WeekDay::Monday);
        let days_in_month = year.number_of_days_in_month(month);

        let mut init = 0;
        if day_before_first_monday != 0 {
            result.push((1, 1..=day_before_first_monday));
            init = 1;
        }

        result.extend(
            (day_before_first_monday + 1..=days_in_month)
                .step_by(7)
                .map_with(init + 1, move |day, week_start| {
                    (
                        (week_start, day..=min!(day + 6, days_in_month)),
                        week_start + 1,
                    )
                }),
        );

        result
    }

    #[test]
    fn test_week_start_end() {
        for year in Year::new(2000)..=Year::new(2022) {
            for month in Month::months() {
                for (_, days) in iter_weeks(year, month) {
                    let week_start = Date::new(year, month, *days.start()).unwrap();
                    let week_end = Date::new(year, month, *days.end()).unwrap();

                    for day in days.into_iter().map(|d| Date::new(year, month, d).unwrap()) {
                        assert_eq!(day.week_start(), week_start, "week_start of day: {}", day);
                        assert_eq!(day.week_end(), week_end, "week_end of day: {}", day);
                    }
                }
            }
        }
    }

    #[test]
    fn test_week_number() {
        let year = Year::new(2022);
        let month = Month::November;

        test_week_number_value(year, month, 1, 1..=6);
        test_week_number_value(year, month, 2, 7..=13);
        test_week_number_value(year, month, 3, 14..=20);
        test_week_number_value(year, month, 4, 21..=27);
        test_week_number_value(year, month, 5, 28..=30);

        let year = Year::new(2022);
        let month = Month::December;

        test_week_number_value(year, month, 1, 1..=4);
        test_week_number_value(year, month, 2, 5..=11);
        test_week_number_value(year, month, 3, 12..=18);
        test_week_number_value(year, month, 4, 19..=25);
        test_week_number_value(year, month, 5, 26..=31);

        let year = Year::new(2021);
        let month = Month::November;

        test_week_number_value(year, month, 1, 1..=7);
        test_week_number_value(year, month, 2, 8..=14);
        test_week_number_value(year, month, 3, 15..=21);
        test_week_number_value(year, month, 4, 22..=28);
        test_week_number_value(year, month, 5, 29..=30);
    }

    #[test]
    fn test_week_number_elaborate() {
        for year in Year::new(1990)..=Year::new(2030) {
            for month in Month::months() {
                for (week_number, week) in iter_weeks(year, month) {
                    test_week_number_value(year, month, week_number, week);
                }
            }
        }
    }
}
