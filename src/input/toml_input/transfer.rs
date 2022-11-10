use std::ops::{Add, AddAssign};
use std::time::Duration;

use serde::Deserialize;

use crate::input::Sign;
use crate::time::WorkingDuration;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Transfer {
    previous_month: WorkingDuration,
    next_month: WorkingDuration,
}

impl Transfer {
    pub fn new(previous_month: WorkingDuration, next_month: WorkingDuration) -> Self {
        Self {
            previous_month,
            next_month,
        }
    }

    pub fn previous_month(&self) -> &WorkingDuration {
        &self.previous_month
    }

    pub fn next_month(&self) -> &WorkingDuration {
        &self.next_month
    }

    fn net_transfer(&self) -> (Sign, Duration) {
        let prev = self.previous_month().to_duration();
        let succ = self.next_month().to_duration();

        if prev > succ {
            (Sign::Negative, prev - succ)
        } else {
            (Sign::Positive, succ - prev)
        }
    }
}

impl Add<Transfer> for Duration {
    type Output = Self;

    fn add(self, rhs: Transfer) -> Self::Output {
        let (sign, net_transfer) = rhs.net_transfer();

        // TODO: this may panic for underflow/overflow
        if sign == Sign::Positive {
            self + net_transfer
        } else {
            self - net_transfer
        }
    }
}

impl AddAssign<Transfer> for Duration {
    fn add_assign(&mut self, rhs: Transfer) {
        *self = *self + rhs;
    }
}
