use std::fmt;

use log::{debug, info};
use serde::Deserialize;

use crate::input::json_input::Entry;
use crate::input::scheduler::{DefaultScheduler, SchedulerOptions, Strategy};
use crate::input::scheduler::{ScheduledTime, WorkSchedule};
use crate::input::strategy::{
    self, FirstComeFirstServe, PeekableStrategy, Proportional, Strategy as _,
};
use crate::input::{Month, Task, Transfer};
use crate::time::{Date, TimeStamp, WorkingDuration};
use crate::utils::MapEntry;
use crate::utils::{self, ArrayVec};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum DynamicEntryInput {
    Flex { flex: usize },
    Fixed { duration: WorkingDuration },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DynamicEntry {
    #[serde(default)]
    action: String,
    #[serde(flatten)]
    input: DynamicEntryInput,
    #[serde(default)]
    pause: Option<WorkingDuration>,
    #[serde(default)]
    start: Option<TimeStamp>,
    #[serde(skip)]
    skip_dates: ArrayVec<Date, 31>,
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

    pub fn transfer_time(&self) -> Transfer {
        self.transfer_time
    }
}

impl DynamicEntry {
    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }

    #[must_use]
    pub fn to_entry(&self, start: TimeStamp, time: ScheduledTime) -> Entry {
        let start = self.start.unwrap_or(start);
        Entry::new(
            self.action(),
            time.date().day(),
            start,
            start + time.duration() + self.pause.unwrap_or_default(),
            self.pause,
        )
    }

    #[must_use]
    pub fn with_skip_dates(mut self, dates: ArrayVec<Date, 31>) -> Self {
        self.skip_dates = dates;
        self
    }

    #[must_use]
    pub fn to_task(&self) -> Task {
        match self.input {
            DynamicEntryInput::Fixed { duration } => {
                Task::new_duration(duration).with_filter(self.skip_dates)
            }
            DynamicEntryInput::Flex { flex } => Task::new_flex(flex).with_filter(self.skip_dates),
        }
    }

    pub fn distribute<Id: Copy + fmt::Debug + 'static>(
        // an iterator of the durations how long each entry is and a unique id
        entries: impl Iterator<Item = (Id, Task)>,
        month: &Month,
        options: &SchedulerOptions,
    ) -> ScheduledDistribution<Id> {
        let mut result = Vec::new();

        let remaining_time = {
            let transfer = month.remaining_time();
            if transfer.is_positive() {
                info!(
                    "fixed entries ({}) exceed the month's working time ({})",
                    transfer.next(),
                    month.expected_working_duration()
                );

                return ScheduledDistribution::new(transfer, result, entries.collect());
            } else {
                transfer.previous()
            }
        };

        let mut entries = entries.collect::<Vec<_>>();

        // resolve the duration of the flex entries

        let mut flex_entries = entries
            .iter()
            .filter_map(|(_, task)| task.flex())
            .collect::<Vec<_>>();

        let mut remaining_time_for_flex = remaining_time;

        for (_, task) in entries.iter() {
            if task.flex().is_none() {
                remaining_time_for_flex = remaining_time_for_flex.saturating_sub(task.duration());
            }
        }

        let remainder = utils::divide_proportionally(
            remaining_time_for_flex.as_mins() as usize,
            &mut flex_entries,
        );

        // for now the first entry gets the remainder:
        if let Some(flex) = flex_entries.first_mut() {
            *flex += remainder;
        }

        // the order remains, so update all tasks:
        for (_, task) in entries.iter_mut() {
            if task.flex().is_some() {
                task.resolve_flex(WorkingDuration::from_mins(flex_entries.remove(0) as u16));
            }
        }

        debug!(
            "remaining time for flex {} of {}",
            remaining_time_for_flex, remaining_time
        );

        let mut scheduler = DefaultScheduler::new(month, options);
        let strategy: Box<dyn strategy::Strategy<Id>> = {
            match options.strategy {
                Strategy::FirstComeFirstServe => Box::new(FirstComeFirstServe::new(entries)),
                Strategy::Proportional => Box::new(Proportional::new(entries, remaining_time)),
            }
        };

        let mut strategy = PeekableStrategy::new(strategy);

        for (_, week_dates) in month.year().iter_weeks_in(month.month()) {
            let schedule = WorkSchedule::new(*week_dates.start(), *week_dates.end());

            let scheduled_tasks = schedule.schedule(&mut strategy, &mut scheduler, |date| {
                month.working_time_on_day(date)
            });

            result.extend(scheduled_tasks);
        }

        ScheduledDistribution {
            transfer_time: scheduler.transfer_time(),
            schedule: result,
            remaining: strategy.to_remaining(),
        }
    }
}

impl<'de> MapEntry<'de> for DynamicEntry {
    type Key = String;
    type Value = Self;

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        value.action = key;
        value
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use super::*;

    use crate::input::json_input;
    use crate::input::toml_input;
    use crate::{date, transfer, working_duration};

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    struct EntrySections {
        #[serde(default, deserialize_with = "crate::utils::deserialize_map_entry")]
        pub entry: Vec<DynamicEntry>,
    }

    #[test]
    fn test_deserialize_flex() {
        assert_eq!(
            toml::from_str::<EntrySections>(concat!("[entry.\"first example\"]\n", "flex = 1\n",)),
            Ok(EntrySections {
                entry: vec![DynamicEntry {
                    action: "first example".to_string(),
                    input: DynamicEntryInput::Flex { flex: 1 },
                    pause: None,
                    start: None,
                    skip_dates: ArrayVec::new(),
                }]
            })
        );
    }

    #[test]
    fn test_deserialize_fixed() {
        assert_eq!(
            toml::from_str::<EntrySections>(concat!(
                "[entry.\"another example\"]\n",
                "duration = \"41:23\"\n",
            )),
            Ok(EntrySections {
                entry: vec![DynamicEntry {
                    action: "another example".to_string(),
                    input: DynamicEntryInput::Fixed {
                        duration: working_duration!(41:23)
                    },
                    pause: None,
                    start: None,
                    skip_dates: ArrayVec::new(),
                }]
            }),
        );
    }

    fn month(input: toml_input::Month, working_duration: WorkingDuration) -> Month {
        Month::new(
            input.general().month(),
            input.general().year(),
            input.transfer().unwrap_or_default(),
            input.entries().map(json_input::Entry::from).collect(),
            input.dynamic_entries().cloned().collect(),
            Some(working_duration),
            input.absences().map(|(k, v)| (k, v.clone())).collect(),
            SchedulerOptions {
                daily_limit: working_duration!(06:00),
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_make_dynamic_entries() {
        let month_input: toml_input::Month = toml::from_str(concat!(
            "[general]\n",
            "month = 7\n",
            "year = 2022\n",
            "department = \"MENSA\"\n",
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

        for entries in month.dynamic_entries() {
            ids.insert(entries.action().to_string(), next_id);
            next_id += 1;
        }

        let durations = month
            .dynamic_entries()
            .map(|entry| (ids[entry.action()], entry.to_task()));

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
            DynamicEntry::distribute(durations, &month, &Default::default()),
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
            "department = \"MENSA\"\n",
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

        let mut ids = HashMap::new();
        let mut next_id = 0;

        for entries in month.dynamic_entries() {
            ids.insert(entries.action().to_string(), next_id);
            next_id += 1;
        }

        let durations = month
            .dynamic_entries()
            .map(|entry| (ids[entry.action()], entry.to_task()));

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
            DynamicEntry::distribute(durations, &month, &Default::default()),
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
                vec![(1, Task::new_duration(working_duration!(05:21)))]
            )
        );
    }
}
