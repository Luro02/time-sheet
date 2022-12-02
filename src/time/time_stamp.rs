use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

use derive_more::Display;
use serde::{de, ser, Deserialize, Serialize};
use thiserror::Error;

use crate::time::WorkingDuration;
use crate::utils;
use crate::{max, min, unreachable_unchecked};

#[macro_export]
macro_rules! time_stamp {
    ( $left:literal : $right:literal ) => {{
        static_assertions::const_assert!($left < 24);
        static_assertions::const_assert!($right < 60);

        unsafe { $crate::time::TimeStamp::new_unchecked($left, $right) }
    }};
}

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display(fmt = "{:02}:{:02}", hour, minute)]
pub struct TimeStamp {
    hour: u8,
    minute: u8,
}

#[derive(Debug, Clone, Error, PartialEq)]
#[error("Time is not valid: {hour:02}:{minute:02}")]
pub struct InvalidTime {
    hour: u8,
    minute: u8,
}

impl TimeStamp {
    #[must_use]
    pub fn new(hour: u8, minute: u8) -> Result<Self, InvalidTime> {
        if hour > 23 || minute > 59 {
            return Err(InvalidTime { hour, minute });
        }

        Ok(Self { hour, minute })
    }

    #[doc(hidden)]
    #[must_use]
    pub unsafe fn new_unchecked(hour: u8, minute: u8) -> Self {
        if hour > 23 || minute > 59 {
            unreachable_unchecked!("hour and minute must be in range but are not");
        }

        Self { hour, minute }
    }

    // the maximum TimeStamp is 23:59, which would be 23 * 60 + 59 = 1439
    // u16::MAX is 2^16 - 1 = 65535
    #[must_use]
    const fn as_minutes(&self) -> u16 {
        self.hour as u16 * 60 + self.minute as u16
    }

    // TODO: how about reverse?
    pub const fn elapsed(&self, other: &Self) -> Duration {
        let minutes = max!(self.as_minutes(), other.as_minutes())
            - min!(self.as_minutes(), other.as_minutes());

        Duration::from_secs(minutes as u64 * 60)
    }
}

impl Into<Duration> for TimeStamp {
    fn into(self) -> Duration {
        Duration::from_secs(self.as_minutes() as u64 * 60)
    }
}

impl From<WorkingDuration> for TimeStamp {
    fn from(duration: WorkingDuration) -> Self {
        Self {
            hour: duration.hours(),
            minute: duration.minutes(),
        }
    }
}

impl From<Duration> for TimeStamp {
    fn from(duration: Duration) -> Self {
        let minutes = duration.as_secs() / 60;

        Self::new(((minutes / 60) % 24) as u8, (minutes % 60) as u8).unwrap()
    }
}

impl FromStr for TimeStamp {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (hour, minute) = string.split_once(':').unwrap();

        Ok(Self::new(hour.parse()?, minute.parse()?)?)
    }
}

// TODO: delegate by using attribute
impl<'de> Deserialize<'de> for TimeStamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for TimeStamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl Add<Duration> for TimeStamp {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        let seconds = duration.as_secs();
        let minutes = seconds % 60;
        let hours = seconds / 60;

        Self {
            minute: utils::overflowing_add(self.minute as u64, minutes, 60) as u8,
            hour: utils::overflowing_add(self.hour as u64, hours, 24) as u8,
        }
    }
}

impl Add<WorkingDuration> for TimeStamp {
    type Output = Self;

    fn add(self, duration: WorkingDuration) -> Self::Output {
        let minutes = self.minute as u64 + duration.minutes() as u64;
        let hours = self.hour + duration.hours() + (minutes / 60) as u8;

        Self {
            minute: (minutes % 60) as u8,
            hour: hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_from_duration() {
        // TODO: more tests
        assert_eq!(
            TimeStamp::from(Duration::from_secs(0)),
            TimeStamp::new(0, 0).unwrap()
        );

        // this tests overflowing duration:
        assert_eq!(
            TimeStamp::from(Duration::from_secs(144000)),
            TimeStamp::new(16, 0).unwrap()
        );
    }
}
