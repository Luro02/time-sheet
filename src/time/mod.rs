use std::fmt;
use std::time::Duration;

#[must_use]
pub fn format_duration(duration: &Duration) -> String {
    PrettyDuration::from(*duration).to_string()
}

pub const fn duration_from_hours(hours: u64) -> Duration {
    Duration::from_secs(hours * 3600)
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrettyDuration(Duration);

impl fmt::Display for PrettyDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.0.as_hours(),
            self.0.as_mins() % 60,
            self.0.as_secs() % 60
        )
    }
}

impl From<Duration> for PrettyDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

pub trait DurationExt {
    #[must_use]
    fn from_hours(hours: u64) -> Duration {
        Self::from_mins(hours * 60)
    }

    #[must_use]
    fn from_mins(mins: u64) -> Duration;

    #[must_use]
    fn as_secs(&self) -> u64;

    #[must_use]
    fn as_mins(&self) -> u64 {
        self.as_secs() / 60
    }

    #[must_use]
    fn as_hours(&self) -> u64 {
        self.as_mins() / 60
    }
}

impl DurationExt for Duration {
    fn from_mins(mins: u64) -> Duration {
        Duration::from_secs(mins * 60)
    }

    fn as_secs(&self) -> u64 {
        Duration::as_secs(self)
    }
}

mod date;
mod month;
mod time_span;
mod time_stamp;
mod week_day;
mod working_duration;
mod year;

pub use date::*;
pub use month::*;
pub use time_span::*;
pub use time_stamp::*;
pub use week_day::*;
pub use working_duration::*;
pub use year::*;
