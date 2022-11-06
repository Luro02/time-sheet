use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

use derive_more::Display;
use serde::{de, ser, Deserialize, Serialize};
use thiserror::Error;

use crate::time::DurationExt;
use crate::utils;

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    #[must_use]
    pub fn new(hours: u8, minutes: u8) -> Result<Self, InvalidWorkingDuration> {
        if hours > 99 || minutes > 99 {
            return Err(InvalidWorkingDuration { hours, minutes });
        }

        Ok(Self { hours, minutes })
    }

    // the maximum WorkingDuration is 99:99, which would be 99 * 60 + 99 = 6039
    // u16::MAX is 2^16 - 1 = 65535
    #[must_use]
    fn as_minutes(&self) -> u16 {
        self.hours as u16 * 60 + self.minutes as u16
    }

    pub fn to_duration(&self) -> Duration {
        Duration::from_mins(self.as_minutes() as u64)
    }
}

impl Into<Duration> for WorkingDuration {
    fn into(self) -> Duration {
        self.to_duration()
    }
}

impl From<Duration> for WorkingDuration {
    fn from(duration: Duration) -> Self {
        let minutes = duration.as_secs() / 60;

        Self::new(((minutes / 60) % 24) as u8, (minutes % 60) as u8).unwrap()
    }
}

impl Default for WorkingDuration {
    fn default() -> Self {
        Self {
            hours: 0,
            minutes: 0,
        }
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
        let seconds = duration.as_secs();
        let minutes = seconds % 60;
        let hours = seconds / 60;

        Self {
            minutes: utils::overflowing_add(self.minutes as u64, minutes, 99) as u8,
            hours: utils::overflowing_add(self.hours as u64, hours, 99) as u8,
        }
    }
}
