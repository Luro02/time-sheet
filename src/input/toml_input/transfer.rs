use serde::Deserialize;

use crate::time::WorkingDuration;

#[derive(Debug, Clone, Deserialize)]
pub struct Transfer {
    previous_month: WorkingDuration,
    next_month: WorkingDuration,
}

impl Transfer {
    pub fn previous_month(&self) -> &WorkingDuration {
        &self.previous_month
    }

    pub fn next_month(&self) -> &WorkingDuration {
        &self.next_month
    }
}
