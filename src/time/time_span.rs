use std::time::Duration;

use derive_more::Display;

use crate::time::TimeStamp;
use crate::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
#[display(fmt = "{} - {}", start, end)]
pub struct TimeSpan {
    start: TimeStamp,
    end: TimeStamp,
}

impl TimeSpan {
    pub fn new(start: TimeStamp, end: TimeStamp) -> Self {
        Self { start, end }
    }

    pub fn overlapping_duration(&self, other: &TimeSpan) -> Option<Duration> {
        // 06:00 to 23:00
        // 03:00 to 07:00
        // -> 01:00

        if self.end < other.start || self.start > other.end {
            return None;
        }

        let overlap_window_start = max!(self.start, other.start);
        let overlap_window_end = min!(self.end, other.end);

        Some(overlap_window_start.elapsed(&overlap_window_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::working_duration;

    #[test]
    fn test_overlapping_duration() {
        // TODO: more tests
        assert_eq!(
            TimeSpan::new(
                TimeStamp::new(6, 0).unwrap(),
                TimeStamp::new(23, 0).unwrap()
            )
            .overlapping_duration(&TimeSpan::new(
                TimeStamp::new(6, 0).unwrap(),
                TimeStamp::new(23, 0).unwrap()
            )),
            Some(Duration::from_secs((23 - 6) * 60 * 60))
        );

        // 06:00 to 23:00
        // 03:00 to 07:00
        // -> 01:00
        assert_eq!(
            TimeSpan::new(
                TimeStamp::new(6, 0).unwrap(),
                TimeStamp::new(23, 0).unwrap()
            )
            .overlapping_duration(&TimeSpan::new(
                TimeStamp::new(3, 0).unwrap(),
                TimeStamp::new(7, 0).unwrap()
            )),
            Some(Duration::from_secs(1 * 60 * 60))
        );

        assert_eq!(
            TimeSpan::new(
                TimeStamp::new(0, 0).unwrap(),
                TimeStamp::new(23, 0).unwrap()
            )
            .overlapping_duration(&TimeSpan::new(
                TimeStamp::new(6, 0).unwrap(),
                TimeStamp::new(23, 0).unwrap()
            )),
            Some(Duration::from_secs((23 - 6) * 60 * 60))
        );

        assert_eq!(
            TimeSpan::new(
                TimeStamp::new(0, 0).unwrap(),
                TimeStamp::new(11, 0).unwrap()
            )
            .overlapping_duration(&TimeSpan::new(
                TimeStamp::new(12, 0).unwrap(),
                TimeStamp::new(23, 0).unwrap()
            )),
            None
        );

        assert_eq!(
            TimeSpan::new(
                TimeStamp::new(0, 0).unwrap(),
                TimeStamp::new(1, 10).unwrap()
            )
            .overlapping_duration(&TimeSpan::new(
                TimeStamp::new(0, 0).unwrap(),
                TimeStamp::new(10, 10).unwrap()
            )),
            Some(working_duration!(01:10).to_duration()),
        );
    }
}
