mod daily_limiter;
mod fixed_scheduler;
mod month_scheduler;
mod time_span;
mod work_schedule;
mod workday_scheduler;

pub use daily_limiter::*;
pub use fixed_scheduler::*;
pub use month_scheduler::*;
pub use time_span::*;
pub use work_schedule::*;
pub use workday_scheduler::*;

use crate::time::{Date, WorkingDuration};

pub trait Scheduler {
    /// Returns the duration that can be worked on that date.
    ///
    /// If there is no more work possible, then `working_duration!(00:00)`
    /// is returned.
    #[must_use]
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration;

    /// Updates the scheduler with the duration that has been worked on that date.
    fn schedule(&mut self, date: Date, worked: WorkingDuration) {
        let _ = date;
        let _ = worked;
    }

    /// Updates the scheduler with the duration that has been worked on that date, but does
    /// not transfer remaining work time.
    fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
        let _ = date;
        let _ = worked;
    }
}

impl<A: Scheduler> Scheduler for &mut A {
    fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
        (**self).has_time_for(date, wanted_duration)
    }

    fn schedule(&mut self, date: Date, worked: WorkingDuration) {
        (**self).schedule(date, worked)
    }

    fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
        (**self).schedule_in_advance(date, worked)
    }
}

macro_rules! impl_scheduler_for_tuple {
    ( $f:ident => $i:tt ) => {
        impl<A: Scheduler> Scheduler for (A,) {
            fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
                self.0.has_time_for(date, wanted_duration)
            }

            fn schedule(&mut self, date: Date, worked: WorkingDuration) {
                self.0.schedule(date, worked);
            }

            fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
                self.0.schedule_in_advance(date, worked);
            }
        }
    };
    ( $f:ident => $i:tt $(, $g:ident => $ig:tt )+ $(,)? ) => {
        impl<$f : Scheduler $(, $g : Scheduler )*> Scheduler for ($f $(, $g)*) {
            fn has_time_for(&self, date: Date, wanted_duration: WorkingDuration) -> WorkingDuration {
                let mut result = wanted_duration;

                result = self.$i.has_time_for(date, result);
                $(
                    result = self.$ig.has_time_for(date, result);
                )*

                result
            }

            fn schedule(&mut self, date: Date, worked: WorkingDuration) {
                self.$i.schedule(date, worked);
                $(
                    self.$ig.schedule(date, worked);
                )*
            }

            fn schedule_in_advance(&mut self, date: Date, worked: WorkingDuration) {
                self.$i.schedule_in_advance(date, worked);
                $(
                    self.$ig.schedule_in_advance(date, worked);
                )*
            }
        }

        impl_scheduler_for_tuple!( $( $g => $ig ),* );
    };
}

impl_scheduler_for_tuple! {
    L => 11,
    K => 10,
    J => 9,
    I => 8,
    H => 7,
    G => 6,
    F => 5,
    E => 4,
    D => 3,
    C => 2,
    B => 1,
    A => 0,
}
