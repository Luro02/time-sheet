mod about;
mod contract;
mod dynamic;
mod entry;
mod entry_key;
mod general;
mod global;
mod month;
mod signature;
mod transfer;

pub use about::*;
pub use contract::*;
pub use dynamic::*;
pub use entry::*;
pub use entry_key::*;
pub use general::*;
pub use global::*;
pub use month::*;
pub use signature::*;
pub use transfer::*;

#[macro_export]
macro_rules! working_duration {
    ( $left:literal : $right:literal ) => {{
        static_assertions::const_assert!($left % 100 == $left);
        static_assertions::const_assert!($right % 100 == $right);
        $crate::time::WorkingDuration::new($left, $right).unwrap()
    }};
}
