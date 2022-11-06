use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum WeekDay {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl WeekDay {
    #[must_use]
    fn week_days() -> [Self; 7] {
        [
            Self::Monday,
            Self::Tuesday,
            Self::Wednesday,
            Self::Thursday,
            Self::Friday,
            Self::Saturday,
            Self::Sunday,
        ]
    }

    pub fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl Add<usize> for WeekDay {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::try_from((self.as_usize() - 1 + rhs % 7) % 7 + 1)
            .expect("WeekDay::try_from is broken")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidWeekDayNumber;

impl TryFrom<usize> for WeekDay {
    type Error = InvalidWeekDayNumber;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::week_days()
            .into_iter()
            .find(|v| *v as usize == value)
            .ok_or(InvalidWeekDayNumber)
    }
}
