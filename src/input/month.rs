use log::debug;
use serde::ser;
use serde::Serialize;

use crate::input::json_input::{Entry, MonthFile};
use crate::input::scheduler::SchedulerOptions;
use crate::input::toml_input::{Absence, DynamicEntry, Holiday, Transfer};
use crate::input::Task;
use crate::time::{self, Date, TimeSpan, TimeStamp, WorkingDuration, Year};
use crate::{time_stamp, working_duration};

#[derive(Debug, Clone)]
pub struct Month {
    year: Year,
    month: time::Month,
    dynamic_entries: Vec<DynamicEntry>,
    expected_working_duration: Option<WorkingDuration>,
    transfer: Transfer,
    entries: Vec<Entry>,
    absence: Vec<(Date, Absence)>,
    options: SchedulerOptions,
}

impl Month {
    const MAXIMUM_WORK_DURATION: WorkingDuration = working_duration!(08:00);
    const DEFAULT_START: TimeStamp = time_stamp!(10:00);

    #[must_use]
    pub fn new(
        month: time::Month,
        year: Year,
        transfer: Transfer,
        entries: Vec<Entry>,
        dynamic_entries: Vec<DynamicEntry>,
        expected_working_duration: Option<WorkingDuration>,
        absence: Vec<(Date, Absence)>,
        options: SchedulerOptions,
    ) -> Self {
        Self {
            month,
            year,
            transfer,
            entries,
            dynamic_entries,
            expected_working_duration,
            absence,
            options,
        }
    }

    pub fn add_entry_if_possible(&mut self, entry: Entry) {
        let span = entry.time_span();
        let entry_date = Date::new(self.year, self.month, entry.day()).unwrap();
        // TODO: is span.duration() right? This would include pauses
        let scheduled = self.schedule(
            Task::new_duration(span.duration())
                .with_start(span.start())
                .with_suggested_date(entry_date),
        );

        if let Some((date, span)) = scheduled.get(0) {
            if *date == entry_date && *span == entry.time_span() {
                self.entries.push(entry);
            }
        }
    }

    /// Returns the amount of time that is remaining to be worked in this month.
    ///
    /// If the remaining time is positive, the working time exceeds the expected working
    /// duration.
    pub fn remaining_time(&self) -> Transfer {
        let fixed_work_duration = self
            .entries
            .iter()
            .map(|e| e.work_duration())
            .sum::<WorkingDuration>();

        Transfer::new(self.expected_working_duration(), fixed_work_duration).normalized()
    }

    /// Finds a free spot where the task can be placed.
    /// In case the task must be split up, multiple spots will be returned.
    fn schedule(&self, task: Task) -> Vec<(Date, TimeSpan)> {
        let mut result = Vec::new();

        let start = task.suggested_start().unwrap_or(Self::DEFAULT_START);
        let mut iter = self.days_with_time_for(task.duration(), Some(start));

        let first = iter.next().expect("No free spot found for task!");

        if let Some(date) = task.suggested_date() {
            if date == first || iter.find(|d| *d == date).is_some() {
                result.push((date, TimeSpan::new(start, start + task.duration())));
                return result;
            }
        } else {
            result.push((first, TimeSpan::new(start, start + task.duration())));
        }

        // TODO: should one implement splitting up the task?

        result
    }

    pub fn schedule_holiday(&mut self, holiday: &Holiday) {
        self.entries.extend(holiday.to_entry(
            self.year,
            self.month,
            self.real_expected_working_duration(),
            |task| self.schedule(task),
        ));
    }

    /// Returns the amount of time that the user should have worked in this month.
    ///
    /// For example if the user has to work 40 hours a month, then there will be
    /// a working time of 40 hours returned.
    #[must_use]
    pub fn expected_working_duration(&self) -> WorkingDuration {
        self.real_expected_working_duration() + self.transfer
    }

    #[must_use]
    pub fn real_expected_working_duration(&self) -> WorkingDuration {
        self.expected_working_duration
            .unwrap_or(working_duration!(40:00))
    }

    pub fn dynamic_entries(&self) -> impl Iterator<Item = &DynamicEntry> {
        self.dynamic_entries.iter()
    }

    #[must_use]
    pub fn year(&self) -> Year {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> time::Month {
        self.month
    }

    /// Returns the amount of time that has been worked in the month.
    /// This includes the time from the previous/next month, which is
    /// specified through the `Transfer`.
    #[must_use]
    pub fn total_working_time(&self) -> WorkingDuration {
        // add the time from the previous/next month
        self.entries
            .iter()
            .map(|e| e.work_duration())
            .sum::<WorkingDuration>()
            + self.transfer.previous()
    }

    pub fn working_time_on_day(&self, date: Date) -> WorkingDuration {
        self.entries_on_day(date)
            .map(|e| e.work_duration())
            .sum::<WorkingDuration>()
    }

    /// Checks if one can work the provided `duration` on that `date`, without exceeding
    /// the maximum working time on that week/day/month.
    fn exceeds_working_duration_on_with(&self, date: Date, duration: WorkingDuration) -> bool {
        let time_on_day: WorkingDuration = self.working_time_on_day(date)
            // add the provided duration to the total working duration
            + duration;

        time_on_day > self.maximum_work_duration()
    }

    #[must_use]
    pub const fn maximum_work_duration(&self) -> WorkingDuration {
        Self::MAXIMUM_WORK_DURATION
    }

    #[must_use]
    fn conflicts_with_existing_entry(&self, date: Date, time_span: TimeSpan) -> bool {
        // check if the time span would exceed the maximum allowed working time
        self.exceeds_working_duration_on_with(date, time_span.duration())
            // check if there is a fixed entry that would overlap with the date/time span
            || self
                .entries_on_day(date)
                .any(|entry| entry.time_span().overlaps_with(time_span))
            // check if there is an absence in that time span
            || self
                .absences_on_day(date)
                .any(|absence| absence.time_span().overlaps_with(time_span))
    }

    fn days_with_time_for(
        &self,
        duration: WorkingDuration,
        start: Option<TimeStamp>,
    ) -> impl Iterator<Item = Date> + '_ {
        self.year()
            .days_in(self.month())
            .filter(move |date| !self.exceeds_working_duration_on_with(*date, duration))
            .filter(move |date| {
                // remove all dates where the start + duration conflict with
                // an existing entry
                start.map_or(true, |start| {
                    !self.conflicts_with_existing_entry(
                        *date,
                        TimeSpan::new(start, start + duration),
                    )
                })
            })
    }

    /// Returns an iterator over all entries that are on the given day.
    fn entries_on_day(&self, date: Date) -> impl Iterator<Item = &Entry> + '_ {
        self.entries
            .iter()
            .filter(move |entry| entry.day() == date.day())
    }

    fn absences_on_day(&self, date: Date) -> impl Iterator<Item = &Absence> + '_ {
        self.absence
            .iter()
            .filter_map(move |(d, absence)| (*d == date).then_some(absence))
    }

    pub fn absence_time_on_day(&self, date: Date) -> WorkingDuration {
        self.absences_on_day(date)
            .map(|absence| absence.duration())
            .sum::<WorkingDuration>()
    }

    /// Returns the transfer time for the month.
    /// (how much time is transfered to the next month/from the previous month)
    #[must_use]
    const fn transfer(&self) -> Transfer {
        self.transfer
    }

    fn to_month_file(&self) -> MonthFile {
        let mut entries = self.entries.clone();

        let mut mapping = Vec::with_capacity(self.dynamic_entries.len());
        let mut durations = Vec::with_capacity(mapping.capacity());

        for dynamic_entry in self.dynamic_entries() {
            let task = dynamic_entry.to_task();
            mapping.push(dynamic_entry);
            durations.push((mapping.len() - 1, task));
        }

        let distribution = DynamicEntry::distribute(durations.into_iter(), self, &self.options);

        debug!("transfer: {:?}", distribution.transfer_time());
        // TODO: what to do with the transfer_tasks and transfer?

        for (id, time) in distribution.schedule() {
            let dynamic_entry = mapping[id];

            entries.push(dynamic_entry.to_entry(Self::DEFAULT_START, time));
        }

        // sort the entries in the json file, so that no problems occur with the java tool
        entries.sort();

        MonthFile::new(self.year, self.month, self.transfer(), entries)
    }

    pub fn actions_that_overflow(&self) -> impl Iterator<Item = &str> + '_ {
        let character_limit = 25;
        self.entries
            .iter()
            .map(|e| e.action())
            .chain(self.dynamic_entries.iter().map(|e| e.action()))
            .filter(move |a| a.len() > character_limit)
    }
}

impl Serialize for Month {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_month_file().serialize(serializer)
    }
}
