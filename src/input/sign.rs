use std::ops::{Mul, MulAssign, Neg, Not};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    pub const fn from_number(number: i32) -> Self {
        if number < 0 {
            Self::Negative
        } else {
            Self::Positive
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            Self::Positive => '+',
            Self::Negative => '-',
        }
    }
}

impl Neg for Sign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }
}

impl Not for Sign {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.neg()
    }
}

impl Mul for Sign {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Positive, Self::Positive) => Self::Positive,
            (Self::Positive, Self::Negative) => Self::Negative,
            (Self::Negative, Self::Positive) => Self::Negative,
            (Self::Negative, Self::Negative) => Self::Positive,
        }
    }
}

impl MulAssign for Sign {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
