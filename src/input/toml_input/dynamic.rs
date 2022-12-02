use std::iter;

use serde::Deserialize;

use crate::input::scheduler::{DailyLimiter, FixedScheduler, MonthScheduler, WorkdayScheduler};
use crate::input::Month;
use crate::input::Scheduler;
use crate::time::{Date, WorkingDuration};
use crate::working_duration;

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

struct WorkSchedule {
    /// The start date of the work schedule.
    start_date: Date,
    /// The end date of the work schedule (inclusive)
    end_date: Date,
}

impl WorkSchedule {
    fn new(start_date: Date, end_date: Date) -> Self {
        Self {
            start_date,
            end_date,
        }
    }

    pub fn schedule<S, P, Id, F>(
        &self,
        mut dynamic_tasks: P,
        mut scheduler: S,
        fixed_scheduler: F,
    ) -> (
        Vec<(Id, Date, WorkingDuration)>,
        Option<(Id, WorkingDuration)>,
    )
    where
        Id: Copy,
        P: Iterator<Item = (Id, WorkingDuration)>,
        S: Scheduler,
        F: Fn(Date) -> WorkingDuration,
    {
        let mut result = Vec::new();

        assert_eq!(self.start_date.year(), self.end_date.year());
        assert_eq!(self.start_date.month(), self.end_date.month());
        let mut current_task = None;

        // schedule fixed tasks in advance
        for date in self.start_date..=self.end_date {
            scheduler.schedule_in_advance(date, fixed_scheduler(date));
        }

        for date in self.start_date..=self.end_date {
            let mut possible_work_duration = scheduler.has_time_for(date, working_duration!(99:59));

            // skips days where no work is possible
            if possible_work_duration == working_duration!(00:00) {
                continue;
            }

            if current_task.is_none() {
                current_task = dynamic_tasks.next();
            }

            // TODO: might enable doing multiple dynamic tasks in one day?
            if let Some((id, task_duration)) = current_task {
                if task_duration <= possible_work_duration {
                    result.push((id, date, task_duration));
                    scheduler.schedule(date, task_duration);
                    // TODO: do more work?
                    possible_work_duration -= task_duration;
                    current_task = None;
                } else {
                    result.push((id, date, possible_work_duration));
                    scheduler.schedule(date, possible_work_duration);
                    current_task.as_mut().unwrap().1 -= possible_work_duration;
                    // possible_work_duration = working_duration!(00:00);
                }
            }
        }

        (result, current_task)
    }
}

impl DynamicEntry {
    #[must_use]
    pub fn duration(&self) -> Option<WorkingDuration> {
        match self.input {
            DynamicEntryInput::Fixed { duration } => Some(duration.into()),
            _ => None,
        }
    }

    pub fn distribute_fixed<Id: Copy>(
        // an iterator of the durations how long each entry is and a unique id
        durations: impl Iterator<Item = (Id, WorkingDuration)>,
        month: &Month,
    ) -> (
        WorkingDuration,
        Vec<(Id, Date, WorkingDuration)>,
        Vec<(Id, WorkingDuration)>,
    ) {
        let mut result = Vec::new();

        let mut iter_dynamic_tasks = durations;
        let mut transfer_task = None;
        let mut month_scheduler = MonthScheduler::new(
            month.year(),
            month.month(),
            month.expected_working_duration(),
        );

        for (_, week_dates) in month.year().iter_weeks_in(month.month()) {
            let schedule = WorkSchedule::new(*week_dates.start(), *week_dates.end());

            let (scheduled_tasks, new_transfer_task) = schedule.schedule(
                iter::from_fn(|| {
                    if let Some((id, duration)) = transfer_task.take() {
                        Some((id, duration))
                    } else {
                        iter_dynamic_tasks.next()
                    }
                }),
                (
                    WorkdayScheduler::new(),
                    FixedScheduler::new(
                        |date| {
                            month
                                .entries_on_day(date)
                                .map(|e| e.work_duration())
                                .sum::<WorkingDuration>()
                        },
                        false,
                    ),
                    DailyLimiter::default(),
                    &mut month_scheduler,
                ),
                |date| {
                    month
                        .entries_on_day(date)
                        .map(|e| e.work_duration())
                        .sum::<WorkingDuration>()
                },
            );

            assert!(transfer_task.is_none() || new_transfer_task.is_none());

            if let Some(new_transfer_task) = new_transfer_task {
                transfer_task = Some(new_transfer_task);
            }

            result.extend(scheduled_tasks);
        }

        let transfer_time = month_scheduler.transfer_time();

        (
            transfer_time.next(), // TODO: prev (remaining time is discarded)
            result,
            transfer_task
                .into_iter()
                .chain(iter_dynamic_tasks.into_iter())
                .collect(),
        )
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
    use crate::{date, working_duration};

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
                .entries()
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
            .map(|(key, entry)| (ids[key], entry.duration().unwrap()));

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
            DynamicEntry::distribute_fixed(durations, &month,),
            (
                working_duration!(00:00),
                vec![
                    // week 1: friday
                    (0, date!(2022:07:01), working_duration!(01:32)), // 01:32
                    // week 2: monday
                    (0, date!(2022:07:04), working_duration!(04:36)), // 06:08
                    // week 3: monday
                    (0, date!(2022:07:11), working_duration!(04:40)), // 10:48
                    // week 4: monday
                    (0, date!(2022:07:18), working_duration!(01:55)), // 12:43
                    // week 4: tuesday
                    (1, date!(2022:07:19), working_duration!(02:41)), // 02:41
                    // week 5: monday
                    (1, date!(2022:07:25), working_duration!(04:36)), // 07:17
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
            .map(|(key, entry)| (ids[key], entry.duration().unwrap()));

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
            DynamicEntry::distribute_fixed(durations, &month),
            (
                working_duration!(00:00),
                vec![
                    // week 1: friday
                    (0, date!(2022:07:01), working_duration!(01:32)), // 01:32
                    // week 2: monday
                    (0, date!(2022:07:04), working_duration!(04:36)), // 06:08
                    // week 4: monday
                    (0, date!(2022:07:18), working_duration!(03:55)), // 10:03
                    // week 5: monday
                    (0, date!(2022:07:25), working_duration!(02:40)), // 12:43
                    // week 5: tuesday
                    (1, date!(2022:07:26), working_duration!(01:56)),
                    // at this point the working limit has been reached
                    // -> the rest must be transferred to the next month
                ],
                vec![(1, working_duration!(05:21)),]
            )
        );
    }
}