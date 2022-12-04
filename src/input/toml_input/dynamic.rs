use std::iter;
use std::ops::{Sub, SubAssign};

use serde::Deserialize;

use crate::input::scheduler::DefaultScheduler;
use crate::input::scheduler::{ScheduledTime, WorkSchedule};
use crate::input::{Month, Transfer};
use crate::time::{Date, WorkingDuration};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum DynamicEntryInput {
    Flex { flex: usize },
    Fixed { duration: WorkingDuration },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DynamicEntry {
    #[serde(flatten)]
    input: DynamicEntryInput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDistribution<Id> {
    transfer_time: Transfer,
    schedule: Vec<(Id, ScheduledTime)>,
    remaining: Vec<(Id, Task)>,
}

impl<Id> ScheduledDistribution<Id> {
    #[must_use]
    pub fn new(
        transfer: Transfer,
        schedule: Vec<(Id, ScheduledTime)>,
        remaining: Vec<(Id, Task)>,
    ) -> Self {
        Self {
            transfer_time: transfer,
            schedule,
            remaining,
        }
    }

    pub fn schedule(self) -> impl IntoIterator<Item = (Id, ScheduledTime)> {
        self.schedule
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Task {
    duration: WorkingDuration,
    suggested_date: Option<Date>,
    can_be_split: bool,
}

impl Task {
    #[must_use]
    pub fn new(
        duration: WorkingDuration,
        suggested_date: Option<Date>,
        can_be_split: bool,
    ) -> Self {
        Self {
            duration,
            suggested_date,
            can_be_split,
        }
    }

    #[must_use]
    pub fn from_duration(duration: WorkingDuration) -> Self {
        Self::new(duration, None, true)
    }

    #[must_use]
    pub fn with_duration(mut self, duration: WorkingDuration) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub fn duration(&self) -> WorkingDuration {
        self.duration
    }

    #[must_use]
    pub fn suggested_date(&self) -> Option<Date> {
        self.suggested_date
    }

    #[must_use]
    pub fn can_be_split(&self) -> bool {
        self.can_be_split
    }
}

impl Sub<WorkingDuration> for Task {
    type Output = Self;

    fn sub(self, rhs: WorkingDuration) -> Self::Output {
        Self::new(self.duration - rhs, self.suggested_date, self.can_be_split)
    }
}

impl SubAssign<WorkingDuration> for Task {
    fn sub_assign(&mut self, rhs: WorkingDuration) {
        self.duration -= rhs;
    }
}

impl DynamicEntry {
    #[must_use]
    pub fn duration(&self) -> Option<WorkingDuration> {
        match self.input {
            DynamicEntryInput::Fixed { duration } => Some(duration),
            _ => None,
        }
    }

    pub fn distribute<Id: Copy>(
        // an iterator of the durations how long each entry is and a unique id
        mut entries: impl Iterator<Item = (Id, Task)>,
        month: &Month,
    ) -> ScheduledDistribution<Id> {
        let mut result = Vec::new();

        let mut transfer_task = None;
        let mut scheduler = DefaultScheduler::new(month);

        for (_, week_dates) in month.year().iter_weeks_in(month.month()) {
            let schedule = WorkSchedule::new(*week_dates.start(), *week_dates.end());
            let dynamic_tasks = iter::from_fn(|| {
                if let Some((id, duration)) = transfer_task.take() {
                    Some((id, duration))
                } else {
                    entries.next()
                }
            });

            let (scheduled_tasks, new_transfer_task) =
                schedule.schedule(dynamic_tasks, &mut scheduler, |date| {
                    month
                        .entries_on_day(date)
                        .map(|e| e.work_duration())
                        .sum::<WorkingDuration>()
                });

            assert!(transfer_task.is_none() || new_transfer_task.is_none());

            if let Some(new_transfer_task) = new_transfer_task {
                transfer_task = Some(new_transfer_task);
            }

            result.extend(scheduled_tasks);
        }

        ScheduledDistribution {
            transfer_time: scheduler.transfer_time(),
            schedule: result,
            remaining: transfer_task.into_iter().chain(entries).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use indexmap::IndexMap;
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::input::json_input;
    use crate::input::toml_input;
    use crate::{date, transfer, working_duration};

    #[derive(Debug, Clone, Deserialize)]
    struct EntrySections {
        pub entry: HashMap<String, DynamicEntry>,
    }

    macro_rules! map {
        ( $( $key:expr => $value:expr ),+ $(,)? ) => {
            {
                let mut _map = ::std::collections::HashMap::new();

                $(
                    _map.insert($key, $value);
                )+

                _map
            }
        };
    }

    #[test]
    fn test_deserialize_flex() {
        let input = concat!("[entry.\"first example\"]\n", "flex = 1\n",);

        let sections: EntrySections = toml::from_str(input).unwrap();

        assert_eq!(
            sections.entry,
            map! {
                "first example".to_string() => DynamicEntry {
                    input: DynamicEntryInput::Flex { flex: 1 },
                },
            }
        );
    }

    #[test]
    fn test_deserialize_fixed() {
        let input = concat!("[entry.\"another example\"]\n", "duration = \"41:23\"\n",);

        let sections: EntrySections = toml::from_str(input).unwrap();

        assert_eq!(
            sections.entry,
            map! {
                "another example".to_string() => DynamicEntry {
                    input: DynamicEntryInput::Fixed { duration: working_duration!(41:23) },
                },
            }
        );
    }

    fn month(input: toml_input::Month, working_duration: WorkingDuration) -> Month {
        Month::new(
            input.general().month(),
            input.general().year(),
            input.transfer().cloned().unwrap_or_default(),
            input
                .entries(working_duration)
                .map(|(key, entry)| json_input::Entry::from((key.clone(), entry.clone())))
                .collect(),
            input
                .dynamic_entries()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            Some(working_duration),
        )
    }

    #[test]
    fn test_make_dynamic_entries() {
        let month_input: toml_input::Month = toml::from_str(concat!(
            "[general]\n",
            "month = 7\n",
            "year = 2022\n",
            "\n",
            "[dynamic.\"wrote python script\"]\n",
            "duration = \"12:43\"\n",
            "\n",
            "[dynamic.\"helped out with grading assignments\"]\n",
            "duration = \"07:17\"\n",
            "\n",
        ))
        .expect("failed to parse input");

        let month = month(month_input, working_duration!(20:00));

        let mut ids = HashMap::new();
        let mut next_id = 0;

        for (key, _) in month.dynamic_entries() {
            ids.insert(key.clone(), next_id);
            next_id += 1;
        }

        let durations = month
            .dynamic_entries()
            .map(|(key, entry)| (ids[key], Task::from_duration(entry.duration().unwrap())));

        // there are no holidays in july and it has 31 days,
        // of those 5 are sundays in 2022 -> 26 working days.
        //
        // A total of 20 hours are available for the month, so
        // each day has 20 * 60 / 26 = 46 minutes available and
        // there is a remainder of 4 minutes.
        //
        // week 1: 2 working days -> 01:32
        // week 2: 6 working days -> 04:36
        // week 3: 6 working days -> 04:36
        // week 4: 6 working days -> 04:36
        // week 5: 6 working days -> 04:36
        //
        // total = 19:56
        // the middle of the month should get the remainder of 4 minutes,
        // -> week 3 would have 04:40
        assert_eq!(
            DynamicEntry::distribute(durations, &month),
            ScheduledDistribution::new(
                transfer!(+00:00),
                vec![
                    // week 1: friday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:01), working_duration!(01:32))
                    ), // 01:32
                    // week 2: monday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:04), working_duration!(04:36))
                    ), // 06:08
                    // week 3: monday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:11), working_duration!(04:40))
                    ), // 10:48
                    // week 4: monday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:18), working_duration!(01:55))
                    ), // 12:43
                    // week 4: tuesday
                    (
                        1,
                        ScheduledTime::new(date!(2022:07:19), working_duration!(02:41))
                    ), // 02:41
                    // week 5: monday
                    (
                        1,
                        ScheduledTime::new(date!(2022:07:25), working_duration!(04:36))
                    ), // 07:17
                ],
                vec![]
            )
        );
    }

    #[test]
    fn test_dynamic_with_transfer() {
        let month_input: toml_input::Month = toml::from_str(concat!(
            "[general]\n",
            "month = 7\n",
            "year = 2022\n",
            "\n",
            "[entries.13]\n",
            "action = \"did prepare presentation\"\n",
            "start = \"10:00\"\n",
            "end = \"15:21\"\n",
            "\n",
            "[dynamic.\"wrote python script\"]\n",
            "duration = \"12:43\"\n",
            "\n",
            "[dynamic.\"helped out with grading assignments\"]\n",
            "duration = \"07:17\"\n",
            "\n",
        ))
        .expect("failed to parse input");

        let month = month(month_input, working_duration!(20:00));

        let mut ids = IndexMap::new();
        let mut next_id = 0;

        for (key, _) in month.dynamic_entries() {
            ids.insert(key.clone(), next_id);
            next_id += 1;
        }

        let durations = month
            .dynamic_entries()
            .map(|(key, entry)| (ids[key], Task::from_duration(entry.duration().unwrap())));

        // there are no holidays in july and it has 31 days,
        // of those 5 are sundays in 2022 -> 26 working days.
        //
        // A total of 20 hours are available for the month, so
        // each day has 20 * 60 / 26 = 46 minutes available and
        // there is a remainder of 4 minutes, which will be given
        // to week 3
        //
        // week 1: 2 working days -> 01:32
        // week 2: 6 working days -> 04:36
        // week 3: 6 working days -> 04:40
        // week 4: 6 working days -> 04:36
        // week 5: 6 working days -> 04:36
        //
        // total = 20:00
        //
        // week 3 has a static entry, which takes up 5:21 hours (00:41 too much)
        // -> the week 4 will only get 04:36 - 00:41 = 03:55
        assert_eq!(
            DynamicEntry::distribute(durations, &month),
            ScheduledDistribution::new(
                transfer!(+00:00),
                vec![
                    // week 1: friday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:01), working_duration!(01:32))
                    ), // 01:32
                    // week 2: monday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:04), working_duration!(04:36))
                    ), // 06:08
                    // week 4: monday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:18), working_duration!(03:55))
                    ), // 10:03
                    // week 5: monday
                    (
                        0,
                        ScheduledTime::new(date!(2022:07:25), working_duration!(02:40))
                    ), // 12:43
                    // week 5: tuesday
                    (
                        1,
                        ScheduledTime::new(date!(2022:07:26), working_duration!(01:56))
                    ),
                    // at this point the working limit has been reached
                    // -> the rest must be transferred to the next month
                ],
                vec![(1, Task::from_duration(working_duration!(05:21)))]
            )
        );
    }
}
