use crate::time::Date;

#[derive(Debug, Clone, PartialEq)]
pub enum CustomEnd {
    /// The event will never stop repeating.
    Never { start: Option<Date> },
    /// The date on which the event ends (inclusive).
    On { start: Option<Date>, end: Date },
    /// The event will stop repeating after `n` repetitions.
    AfterOccurrences { start: Date, count: usize },
}

impl CustomEnd {
    const fn start(&self) -> Option<Date> {
        match self {
            Self::Never { start } => *start,
            Self::On { start, .. } => *start,
            Self::AfterOccurrences { start, .. } => Some(*start),
        }
    }

    fn is_after_start(&self, date: Date) -> bool {
        self.start().map_or(true, |start| start <= date)
    }

    #[must_use]
    pub fn applies_on(&self, date: Date, previous_repetitions: impl FnOnce(Date) -> usize) -> bool {
        match self {
            Self::Never { .. } => self.is_after_start(date),
            Self::On { end, .. } => self.is_after_start(date) && date <= *end,
            Self::AfterOccurrences { start, count } => {
                self.is_after_start(date) && previous_repetitions(*start) < *count
            }
        }
    }
}

impl Default for CustomEnd {
    fn default() -> Self {
        Self::Never { start: None }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Bound, RangeBounds, Sub};

    use super::*;

    use pretty_assertions::assert_eq;

    use crate::date;

    #[test]
    fn test_default() {
        assert_eq!(CustomEnd::default(), CustomEnd::Never { start: None });
    }

    #[track_caller]
    fn check_applies_on(custom_end: &CustomEnd, date: Date, expected: bool) {
        assert_eq!(
            custom_end.applies_on(date, |_| 0),
            expected,
            "{:?} should {}apply on `{}`",
            custom_end,
            {
                if expected {
                    ""
                } else {
                    "not "
                }
            },
            date
        );
    }

    /// Checks that the custom end applies on the given range
    /// and does not apply outside of it.
    #[track_caller]
    fn check_applies_on_range(custom_end: &CustomEnd, range: impl RangeBounds<Date>) {
        fn resolve_bound<T>(bound: Bound<T>, is_start: bool) -> Option<T>
        where
            T: Add<usize, Output = T> + Sub<usize, Output = T>,
        {
            match bound {
                Bound::Unbounded => None,
                Bound::Included(value) => Some(value),
                Bound::Excluded(value) => {
                    if is_start {
                        Some(value - 1_usize)
                    } else {
                        Some(value + 1_usize)
                    }
                }
            }
        }

        let offset = 2 * 365_usize;
        let start_date = resolve_bound(range.start_bound().cloned(), true);
        let end_date = resolve_bound(range.end_bound().cloned(), false);

        let start =
            start_date.unwrap_or_else(|| end_date.map_or(date!(2022:12:31), |end| end - offset));
        let end = end_date
            .unwrap_or_else(|| start_date.map_or(date!(2021:01:01), |start| start + offset));

        // check the range:
        for date in start..=end {
            check_applies_on(custom_end, date, true);
        }

        if let Some(start) = start_date {
            // check before the start date:
            for date in (start - offset)..start {
                check_applies_on(custom_end, date, false);
            }
        }

        if let Some(end) = end_date {
            // check after the end date:
            for date in (end + 1)..(end + 1 + offset) {
                check_applies_on(custom_end, date, false);
            }
        }
    }

    #[test]
    fn test_never_applies_on() {
        check_applies_on_range(&CustomEnd::Never { start: None }, ..);
        check_applies_on_range(
            &CustomEnd::Never {
                start: Some(date!(2022:03:01)),
            },
            date!(2022:03:01)..,
        );

        check_applies_on_range(
            &CustomEnd::Never {
                start: Some(date!(2022:02:05)),
            },
            date!(2022:02:05)..,
        );
    }

    #[test]
    fn test_on_applies_on() {
        check_applies_on_range(
            &CustomEnd::On {
                start: None,
                end: date!(2022:03:01),
            },
            ..=date!(2022:03:01),
        );

        check_applies_on_range(
            &CustomEnd::On {
                start: Some(date!(2021:01:01)),
                end: date!(2022:03:01),
            },
            date!(2021:01:01)..=date!(2022:03:01),
        );

        check_applies_on_range(
            &CustomEnd::On {
                start: Some(date!(2022:03:01)),
                end: date!(2022:03:01),
            },
            date!(2022:03:01)..=date!(2022:03:01),
        );
    }
}
