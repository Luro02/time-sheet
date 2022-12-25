use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use std::str::FromStr;
use std::time::Duration;

use derive_more::Display;
use serde::{de, ser, Deserialize, Serialize};
use thiserror::Error;

use crate::time::DurationExt;
use crate::unreachable_unchecked;

#[macro_export]
macro_rules! working_duration {
    ( $left:literal : $right:literal ) => {{
        static_assertions::const_assert!($left % 100 == $left);
        static_assertions::const_assert!($right % 60 == $right);

        unsafe { $crate::time::WorkingDuration::new_unchecked($left, $right) }
    }};
}

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[display(fmt = "{:02}:{:02}", hours, minutes)]
pub struct WorkingDuration {
    hours: u8,
    minutes: u8,
}

#[derive(Debug, Clone, Error, PartialEq)]
#[error("Duration is not valid: {hours:02}:{minutes:02}")]
pub struct InvalidWorkingDuration {
    hours: u8,
    minutes: u8,
}

impl WorkingDuration {
    pub const fn new(hours: u8, minutes: u8) -> Result<Self, InvalidWorkingDuration> {
        if hours > 99 || minutes > 60 {
            return Err(InvalidWorkingDuration { hours, minutes });
        }

        Ok(Self { hours, minutes })
    }

    // internal non-public api to make the `working_duration` macro work in const context.
    #[doc(hidden)]
    #[must_use]
    pub const unsafe fn new_unchecked(hours: u8, minutes: u8) -> Self {
        if hours > 99 || minutes > 60 {
            unreachable_unchecked!("hours and minutes must be in range 0..=99 but are not");
        }

        Self { hours, minutes }
    }

    #[must_use]
    pub const fn from_mins(mins: u16) -> Self {
        let hours = mins / 60;
        let minutes = mins % 60;

        if hours > 99 {
            panic!("hours must be in range 0..=99");
        }

        unsafe { Self::new_unchecked(hours as u8, minutes as u8) }
    }

    // the maximum WorkingDuration is 99:99, which would be 99 * 60 + 99 = 6039
    // u16::MAX is 2^16 - 1 = 65535
    #[must_use]
    pub const fn as_mins(&self) -> u16 {
        self.hours as u16 * 60 + self.minutes as u16
    }

    pub fn to_duration(&self) -> Duration {
        Duration::from_mins(self.as_mins() as u64)
    }

    pub const fn hours(&self) -> u8 {
        self.hours
    }

    pub const fn minutes(&self) -> u8 {
        self.minutes
    }

    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let mins = self.as_mins().saturating_sub(other.as_mins());

        Self::from_mins(mins)
    }
}

impl From<WorkingDuration> for Duration {
    fn from(value: WorkingDuration) -> Self {
        value.to_duration()
    }
}

impl From<Duration> for WorkingDuration {
    fn from(duration: Duration) -> Self {
        Self::from_mins(duration.as_mins() as u16)
    }
}

impl FromStr for WorkingDuration {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (hour, minute) = string.split_once(':').unwrap();

        Ok(Self::new(hour.parse()?, minute.parse()?)?)
    }
}

// TODO: delegate by using attribute
impl<'de> Deserialize<'de> for WorkingDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for WorkingDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl Add<Duration> for WorkingDuration {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        Self::from_mins((duration.as_mins() + self.as_mins() as u64) as u16)
    }
}

impl AddAssign<Duration> for WorkingDuration {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Add for WorkingDuration {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        self + other.to_duration()
    }
}

impl AddAssign for WorkingDuration {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for WorkingDuration {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::from(self.to_duration() - other.to_duration())
    }
}

impl Mul<u32> for WorkingDuration {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::from(self.to_duration() * rhs)
    }
}

impl SubAssign for WorkingDuration {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<T> Sum<T> for WorkingDuration
where
    Self: Add<T, Output = Self>,
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        iter.fold(Self::default(), Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::working_duration;

    #[test]
    fn test_add_duration() {
        // 12:23:42
        let duration = Duration::from_hours(12) + Duration::from_mins(23) + Duration::from_secs(42);

        assert_eq!(
            working_duration!(01:20) + duration,
            working_duration!(13:43)
        );

        // test with overflow:
        assert_eq!(
            working_duration!(01:40) + duration,
            working_duration!(14:03)
        );

        assert_eq!(
            working_duration!(01:38) * (6 * 3 + 4 + 3) + working_duration!(00:10),
            working_duration!(41:00),
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            working_duration!(01:20) - working_duration!(00:40),
            working_duration!(00:40)
        );

        assert_eq!(
            working_duration!(01:20) - working_duration!(01:20),
            working_duration!(00:00)
        );

        assert_eq!(
            working_duration!(02:20) - working_duration!(01:20),
            working_duration!(01:00)
        );

        // essentially the following property has to hold:
        // (a + b) - b = a
    }
}
