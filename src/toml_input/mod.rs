mod about;
mod contract;
mod entry;
mod entry_key;
mod general;
mod global;
mod month;
mod signature;
mod transfer;

pub use about::*;
pub use contract::*;
pub use entry::*;
pub use entry_key::*;
pub use general::*;
pub use global::*;
pub use month::*;
pub use signature::*;
pub use transfer::*;

/// Taken from `static_assertions` crate
// TODO: might consider using `static_assertions` crate instead
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}

/// Just a small proof of concept that I wrote, because I was bored.
macro_rules! working_duration {
    ( $left:literal : $right:literal ) => {
        const_assert!($left % 100 == $left);
        const_assert!($right % 100 == $right);

        WorkingDuration::new($left, $right).unwrap()
    };
}
