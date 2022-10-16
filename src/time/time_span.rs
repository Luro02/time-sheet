use std::cmp;
use std::time::Duration;

use derive_more::Display;

use crate::time::TimeStamp;

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

        let overlap_window_end = cmp::min(self.end, other.end);
        let overlap_window_start = cmp::max(self.start, other.start);

        if self.end <= other.start && self.start >= other.end {
            return None;
        }

        Some(overlap_window_start.elapsed(&overlap_window_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

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
    }
}
