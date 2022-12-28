use std::str::FromStr;

use serde::Deserialize;

use crate::time::WorkingDuration;
use crate::working_duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(try_from = "String")]
pub enum Strategy {
    #[default]
    FirstComeFirstServe,
    Proportional,
}

impl FromStr for Strategy {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "first-come-first-serve" => Ok(Self::FirstComeFirstServe),
            "proportional" => Ok(Self::Proportional),
            _ => Err(anyhow::anyhow!("Unknown strategy: {}", string)),
        }
    }
}

impl TryFrom<String> for Strategy {
    type Error = <Self as FromStr>::Err;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        Self::from_str(&string)
    }
}

/// Options to configure the default scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SchedulerOptions {
    /// If this is set to `true`, tasks can be scheduled on days where the user
    /// has fixed entries.
    pub should_schedule_with_fixed_entries: bool,
    /// If this is set to `true`, tasks can be scheduled on days where the user
    /// might be absent.
    ///
    /// Otherwise the scheduler will avoid scheduling tasks on those days.
    pub should_schedule_with_absences: bool,
    /// The maximum duration that can be scheduled on a single day.
    pub daily_limit: WorkingDuration,
    /// The strategy to use for scheduling tasks.
    pub strategy: Strategy,
}

impl Default for SchedulerOptions {
    fn default() -> Self {
        Self {
            should_schedule_with_fixed_entries: false,
            should_schedule_with_absences: false,
            daily_limit: working_duration!(06:00),
            strategy: Default::default(),
        }
    }
}
