use std::fmt;
use std::time::Duration;

// TODO: this or PrettyDuration?
pub fn format_duration(duration: &Duration) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        duration.hours(),
        duration.minutes(),
        duration.seconds()
    )
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
            self.0.hours(),
            self.0.minutes() % 60,
            self.0.seconds() % 60
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
    fn seconds(&self) -> u64;

    #[must_use]
    fn minutes(&self) -> u64 {
        self.seconds() / 60
    }

    #[must_use]
    fn hours(&self) -> u64 {
        self.minutes() / 60
    }
}

impl DurationExt for Duration {
    fn from_mins(mins: u64) -> Duration {
        Duration::from_secs(mins * 60)
    }

    fn seconds(&self) -> u64 {
        self.as_secs()
    }
}

mod month;
pub use month::*;
mod date;
pub use date::*;
mod week_day;
pub use week_day::*;
mod year;
pub use year::*;
mod time_stamp;
pub use time_stamp::*;
mod time_span;
pub use time_span::*;
mod working_duration;
pub use working_duration::*;
