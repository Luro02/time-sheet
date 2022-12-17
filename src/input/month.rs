use serde::ser;
use serde::Serialize;

use crate::input::json_input::{Entry, MonthFile};
use crate::input::scheduler::SchedulerOptions;
use crate::input::toml_input::{Absence, DynamicEntry, Holiday, Task, Transfer};
use crate::time::{self, Date, TimeSpan, TimeStamp, WeekDay, WorkingDuration, Year};
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
}

impl Month {
    const MAXIMUM_WORK_DURATION: WorkingDuration = working_duration!(08:00);

    #[must_use]
    pub fn new(
        month: time::Month,
        year: Year,
        transfer: Transfer,
        entries: Vec<Entry>,
        dynamic_entries: Vec<DynamicEntry>,
        expected_working_duration: Option<WorkingDuration>,
        absence: Vec<(Date, Absence)>,
    ) -> Self {
        Self {
            month,
            year,
            transfer,
            entries,
            dynamic_entries,
            expected_working_duration,
            absence,
        }
    }

    pub fn add_entry_if_possible(&mut self, entry: Entry) {
        let span = entry.time_span();
        let entry_date = Date::new(self.year, self.month, entry.day()).unwrap();
        let scheduled = self.schedule(Task::new_with_start(
            span.duration(),
            Some(entry_date),
            false,
            span.start(),
        ));

        if let Some((date, span)) = scheduled.get(0) {
            if *date == entry_date && *span == entry.time_span() {
                self.entries.push(entry);
            }
        }
    }

    /// Finds a free spot where the task can be placed.
    /// In case the task must be split up, multiple spots will be returned.
    fn schedule(&self, task: Task) -> Vec<(Date, TimeSpan)> {
        let mut result = Vec::new();

        let start = task.suggested_start().unwrap_or_else(|| time_stamp!(08:00));
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
            self.expected_working_duration(),
            |task| self.schedule(task),
        ));
    }

    /// Returns the amount of time that the user should have worked in this month.
    ///
    /// For example if the user has to work 40 hours a month, then there will be
    /// a working time of 40 hours returned.
    #[must_use]
    pub fn expected_working_duration(&self) -> WorkingDuration {
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

    /// Checks if one can work the provided `duration` on that `date`, without exceeding
    /// the maximum working time on that week/day/month.
    pub fn exceeds_working_duration_on_with(&self, date: Date, duration: WorkingDuration) -> bool {
        let time_on_day: WorkingDuration = self
            .entries_on_day(date)
            .map(|e| e.work_duration())
            .sum::<WorkingDuration>()
            // add the provided duration to the total working duration
            + duration;

        time_on_day > self.maximum_work_duration()
    }

    #[must_use]
    pub const fn maximum_work_duration(&self) -> WorkingDuration {
        Self::MAXIMUM_WORK_DURATION
    }

    // TODO: make use of this more?
    #[must_use]
    pub fn conflicts_with_existing_entry(&self, date: Date, time_span: TimeSpan) -> bool {
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

    pub fn days_with_time_for(
        &self,
        duration: WorkingDuration,
        start: Option<TimeStamp>,
    ) -> impl Iterator<Item = Date> + '_ {
        self.year()
            .iter_days_in(self.month())
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

    /// Checks whether or not there is work scheduled on the provided date.
    #[must_use]
    pub fn has_entries_on(&self, date: Date) -> bool {
        self.entries_on_day(date).next().is_some()
    }

    /// Returns days in the `month` where no fixed work is planned or has been made.
    pub fn free_days(&self) -> impl Iterator<Item = Date> + '_ {
        self.year()
            .iter_days_in(self.month())
            // skip all sundays, where one is not allowed to work
            .filter(|date| date.week_day() != WeekDay::Sunday)
            // skip all holidays, where one is not allowed to work
            .filter(|date| !date.is_holiday())
            // skip all days where there is already an entry
            .filter(|date| self.entries_on_day(*date).next().is_some())
    }

    /// Returns an iterator over all entries that are on the given day.
    pub fn entries_on_day(&self, date: Date) -> impl Iterator<Item = &Entry> + '_ {
        self.entries
            .iter()
            .filter(move |entry| entry.day() == date.day())
    }

    pub fn absences_on_day(&self, date: Date) -> impl Iterator<Item = &Absence> + '_ {
        self.absence
            .iter()
            .filter_map(move |(d, absence)| (*d == date).then_some(absence))
    }

    /// Returns the transfer time for the month.
    /// (how much time is transfered to the next month/from the previous month)
    #[must_use]
    pub const fn transfer(&self) -> Transfer {
        self.transfer
    }

    fn to_month_file(&self) -> MonthFile {
        let mut entries = self.entries.clone();

        let mut mapping = Vec::with_capacity(self.dynamic_entries.len());
        let mut durations = Vec::with_capacity(mapping.capacity());

        for dynamic_entry in self.dynamic_entries() {
            if let Some(duration) = dynamic_entry.duration() {
                mapping.push(dynamic_entry);
                durations.push((mapping.len() - 1, Task::from_duration(duration)));
            }
        }

        let distribution = DynamicEntry::distribute(
            durations.into_iter(),
            self,
            &SchedulerOptions {
                daily_limit: working_duration!(06:00),
                ..Default::default()
            },
        );

        // TODO: what to do with the transfer_tasks and transfer?

        for (id, time) in distribution.schedule() {
            let dynamic_entry = mapping[id];
            let mut pause = None;
            let duration = time.duration();
            let date = time.date();

            // TODO: make it possible to configure this?
            // TODO: automagically add pauses for too long fixed entries?
            if duration > working_duration!(04:00) {
                pause = Some(working_duration!(00:30));
            }

            let start = time_stamp!(10:00);
            let end = start + duration + pause.unwrap_or_default();

            entries.push(Entry::new(
                dynamic_entry.action().to_string(),
                date.day(),
                start,
                end,
                pause,
            ))
        }

        // sort the entries in the json file, so that no problems occur with the java tool
        entries.sort();

        MonthFile::new(self.year, self.month, self.transfer(), entries)
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
