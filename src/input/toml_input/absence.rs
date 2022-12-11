use serde::Deserialize;

use crate::time::{TimeSpan, TimeStamp, WorkingDuration};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Absence {
    /// When the absence starts on the first day.
    start: TimeStamp,
    /// When the absence ends on the last day.
    end: TimeStamp,
}

impl Absence {
    #[must_use]
    pub const fn time_span(&self) -> TimeSpan {
        TimeSpan::new(self.start, self.end)
    }

    pub const fn duration(&self) -> WorkingDuration {
        self.time_span().duration()
    }
}
