use serde::Deserialize;

use crate::input::toml_input::repeating::{CustomEnd, RepeatsEvery};
use crate::input::toml_input::{DynamicEntry, Entry};
use crate::time::{Date, Month, TimeSpan, TimeStamp, WeekDay, WorkingDuration, Year};
use crate::utils::{ArrayVec, MapEntry};

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRepeatInterval {
    repeats_every: RepeatsEvery,
    repeats_on: [bool; 7],
    end: CustomEnd,
}

impl CustomRepeatInterval {
    pub fn new(repeats_every: RepeatsEvery, end: CustomEnd, repeats_on: Vec<WeekDay>) -> Self {
        // TODO: check if start date is required and correctly set repeats_on?

        Self {
            repeats_every,
            repeats_on: WeekDay::week_days().map(|day| repeats_on.contains(&day)),
            end,
        }
    }

    pub fn repeats_on(&self, date: Date) -> bool {
        self.repeats_on[date.week_day().as_usize() - 1]
            && self
                .end
                .applies_on(date, |start| self.repeats_every.repetitions(start, date))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum RepeatingKind {
    /// Repeats on the given weekdays.
    WeekDays {
        repeats_on: Vec<WeekDay>,
        #[serde(default)]
        end_date: Option<Date>,
    },
    /// Repeats only after the given date.
    FixedStart {
        start_date: Date,
        #[serde(default)]
        end_date: Option<Date>,
    },
    /// Repeats only on the given dates.
    FixedDates { dates: Vec<Date> },
}

impl RepeatingKind {
    pub fn iter_week_days(&self) -> impl Iterator<Item = WeekDay> {
        match self {
            Self::WeekDays { repeats_on, .. } => repeats_on.clone().into_iter(),
            Self::FixedStart { start_date, .. } => vec![start_date.week_day()].into_iter(),
            Self::FixedDates { .. } => vec![].into_iter(),
        }
    }

    const fn end_date(&self) -> Option<Date> {
        match self {
            Self::WeekDays { end_date, .. } => *end_date,
            Self::FixedStart { end_date, .. } => *end_date,
            Self::FixedDates { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum EventKind {
    Dynamic {
        #[serde(flatten)]
        entry: DynamicEntry,
    },
    Normal {
        #[serde(default)]
        action: String,
        start: TimeStamp,
        end: TimeStamp,
        #[serde(default)]
        pause: Option<WorkingDuration>,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RepeatingEvent {
    repeats_every: RepeatsEvery,
    #[serde(flatten)]
    repeating_kind: RepeatingKind,
    #[serde(default)]
    department: Option<String>,
    #[serde(default, rename = "vacation")]
    is_vacation: bool,
    #[serde(flatten)]
    event_kind: EventKind,
}

impl RepeatingEvent {
    fn start(&self) -> Option<Date> {
        match &self.repeating_kind {
            RepeatingKind::WeekDays { .. } => None,
            RepeatingKind::FixedStart { start_date, .. } => Some(*start_date),
            RepeatingKind::FixedDates { dates, .. } => dates.iter().min().copied(),
        }
    }

    fn custom_end(&self) -> CustomEnd {
        let start = self.start();

        let Some(end) = self.repeating_kind.end_date() else {
            return CustomEnd::Never { start };
        };

        CustomEnd::On { start, end }
    }

    #[must_use]
    fn repeats_on(&self, date: Date) -> bool {
        if let RepeatingKind::FixedDates { dates } = &self.repeating_kind {
            return dates.contains(&date);
        }

        CustomRepeatInterval::new(
            self.repeats_every,
            self.custom_end(),
            self.repeating_kind.iter_week_days().collect(),
        )
        .repeats_on(date)
    }

    fn to_dynamic_entry(&self, department: &str) -> Option<DynamicEntry> {
        // If a department is specified, only apply if the department matches
        if self.department.is_some() && self.department.as_deref() != Some(department) {
            return None;
        }

        if let EventKind::Dynamic { entry } = &self.event_kind {
            // update the action of the dynamic entry
            Some(entry.clone())
        } else {
            None
        }
    }

    pub fn to_dynamic_entries(
        &self,
        year: Year,
        month: Month,
        department: &str,
        mut can_repeat_on: impl FnMut(Date) -> bool,
    ) -> impl IntoIterator<Item = DynamicEntry> + '_ {
        let mut entries: ArrayVec<_, 31> = ArrayVec::new();

        let range = Date::first_day(year, month)..=Date::last_day(year, month);

        if let Some(e) = self.to_dynamic_entry(department) {
            for date in range.clone() {
                if can_repeat_on(date) && self.repeats_on(date) {
                    entries.push(
                        e.clone()
                            .with_skip_dates(range.clone().filter(|d| d != &date).collect()),
                    );
                }
            }
        }

        entries
    }

    pub fn to_entry(&self, date: Date, department: &str) -> Option<Entry> {
        if !self.repeats_on(date) {
            return None;
        }

        // If a department is specified, only apply if the department matches
        if self.department.is_some() && self.department.as_deref() != Some(department) {
            return None;
        }

        if let EventKind::Normal {
            action,
            start,
            end,
            pause,
        } = &self.event_kind
        {
            Some(Entry::new(
                date.day(),
                action.to_string(),
                TimeSpan::new(*start, *end),
                *pause,
                Some(self.is_vacation),
            ))
        } else {
            None
        }
    }
}

impl<'de> MapEntry<'de> for RepeatingEvent {
    type Key = String;
    type Value = Self;

    fn new(key: Self::Key, mut value: Self::Value) -> Self {
        match &mut value.event_kind {
            EventKind::Dynamic { entry } => {
                *entry = DynamicEntry::new(key, entry.clone());
            }
            EventKind::Normal { action, .. } => {
                *action = key;
            }
        }

        value
    }
}
