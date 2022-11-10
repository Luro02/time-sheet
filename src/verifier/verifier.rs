use std::fmt;
use std::fmt::Debug;

use crate::input::Config;

// relevant data from MonthFile:
// year + month (for date)
// pred_transfer and succ_transfer <- make this its own function?
// entries <- for how much one has worked?
// working_time <- relevant for the entries verification and pred_transfer / succ_transfer

// => Date, Transfer, Entries, WorkingTime

pub trait Verifier {
    type Error: fmt::Display + Debug + Sync + Send + 'static;
    type Errors: IntoIterator<Item = Self::Error>;

    fn verify(&self, config: &Config) -> Result<(), Self::Errors>;
}
