use core::fmt;
use std::ops::{Add, AddAssign, Mul};
use std::time::Duration;

use serde::Deserialize;

use crate::input::Sign;
use crate::time::WorkingDuration;
use crate::working_duration;

#[macro_export]
macro_rules! transfer {
    (+$left:literal : $right:literal) => {
        $crate::input::Transfer::positive($crate::working_duration!($left: $right))
    };
    (-$left:literal : $right:literal) => {
        $crate::input::Transfer::negative($crate::working_duration!($left: $right))
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct Transfer {
    previous_month: WorkingDuration,
    next_month: WorkingDuration,
}

impl Transfer {
    pub const fn new(previous_month: WorkingDuration, next_month: WorkingDuration) -> Self {
        Self {
            previous_month,
            next_month,
        }
    }

    pub const fn positive(time: WorkingDuration) -> Self {
        Self::new(working_duration!(00:00), time)
    }

    pub const fn negative(time: WorkingDuration) -> Self {
        Self::new(time, working_duration!(00:00))
    }

    pub fn is_positive(&self) -> bool {
        self.next_month >= self.previous_month
    }

    pub const fn previous(&self) -> WorkingDuration {
        self.previous_month
    }

    pub const fn next(&self) -> WorkingDuration {
        self.next_month
    }

    fn net_transfer(&self) -> (Sign, Duration) {
        let prev = self.previous().to_duration();
        let succ = self.next().to_duration();

        if prev > succ {
            (Sign::Negative, prev - succ)
        } else {
            (Sign::Positive, succ - prev)
        }
    }

    fn from_sign(sign: Sign, duration: Duration) -> Self {
        match sign {
            Sign::Positive => Self::positive(WorkingDuration::from(duration)),
            Sign::Negative => Self::negative(WorkingDuration::from(duration)),
        }
    }
}

// TODO: implement for WorkingDuration?
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

impl Add<Transfer> for Transfer {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (self_sign, self_net_transfer) = self.net_transfer();
        let (rhs_sign, rhs_net_transfer) = rhs.net_transfer();

        if self_sign == rhs_sign {
            Self::from_sign(self_sign, self_net_transfer + rhs_net_transfer)
        } else if self_net_transfer >= rhs_net_transfer {
            Self::from_sign(self_sign, self_net_transfer - rhs_net_transfer)
        } else {
            Self::from_sign(rhs_sign, rhs_net_transfer - self_net_transfer)
        }
    }
}

impl Mul<u32> for Transfer {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        let (sign, transfer) = self.net_transfer();
        Self::from_sign(sign, transfer * rhs)
    }
}

impl Mul<i32> for Transfer {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        let (sign, transfer) = self.net_transfer();
        let other_sign = Sign::from_number(rhs);

        Self::from_sign(sign * other_sign, transfer * rhs.unsigned_abs())
    }
}

impl AddAssign<Transfer> for Transfer {
    fn add_assign(&mut self, rhs: Transfer) {
        *self = *self + rhs;
    }
}

impl AddAssign<Transfer> for Duration {
    fn add_assign(&mut self, rhs: Transfer) {
        *self = *self + rhs;
    }
}

impl Add<Transfer> for WorkingDuration {
    type Output = Self;

    fn add(self, transfer: Transfer) -> Self::Output {
        WorkingDuration::from(self.to_duration() + transfer)
    }
}

impl fmt::Debug for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (sign, net_transfer) = self.net_transfer();
        f.debug_tuple("Transfer")
            .field(&format!(
                "{}{}",
                sign.symbol(),
                WorkingDuration::from(net_transfer)
            ))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{duration, working_duration};

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_transfer_macro() {
        assert_eq!(
            transfer!(+1:00),
            Transfer::positive(working_duration!(1:00))
        );
        assert_eq!(
            transfer!(-1:00),
            Transfer::negative(working_duration!(1:00))
        );
    }

    #[test]
    fn test_mul_transfer() {
        assert_eq!(transfer!(-00:29) * 0, transfer!(+00:00));
    }

    #[test]
    fn test_add_transfer_to_transfer() {
        let test_cases = [
            (transfer!(-01:00), transfer!(+01:00), transfer!(+00:00)),
            (transfer!(-02:00), transfer!(+01:00), transfer!(-01:00)),
            (transfer!(-01:25), transfer!(+01:54), transfer!(+00:29)),
            (transfer!(-12:34), transfer!(-12:57), transfer!(-25:31)),
            (transfer!(+12:34), transfer!(+12:57), transfer!(+25:31)),
            (transfer!(+00:00), transfer!(+00:00), transfer!(+00:00)),
            (transfer!(-00:00), transfer!(-00:00), transfer!(-00:00)),
        ];

        for (lhs, rhs, expected) in test_cases {
            assert_eq!(lhs + rhs, expected);
            assert_eq!(rhs + lhs, expected);
        }
    }

    #[test]
    fn test_add_to_duration() {
        assert_eq!(
            duration!(02:31:00) + Transfer::new(working_duration!(01:05), working_duration!(00:00)),
            duration!(01:26:00)
        );

        assert_eq!(
            duration!(02:31:00) + Transfer::new(working_duration!(00:00), working_duration!(01:09)),
            duration!(03:40:00)
        );
    }

    #[test]
    fn test_default() {
        assert_eq!(
            Transfer::default(),
            Transfer::new(working_duration!(00:00), working_duration!(00:00))
        );
    }
}
