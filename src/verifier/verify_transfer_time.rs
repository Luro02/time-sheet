use std::iter;
use std::time::Duration;

use thiserror::Error;

use crate::files::MonthFile;
use crate::time::{DurationExt, PrettyDuration};
use crate::verifier::Verifier;

/// This verifies that the successive transfer time is equal to
/// total_time - expected_working_time and it is >= 0
pub struct VerifyTransferTime;

#[derive(Debug, Clone, Error, PartialEq)]
#[error("invalid transfer time: {total_time} - {expected_working_time} != {transfer_time}")]
pub struct InvalidTransferTime {
    expected_working_time: PrettyDuration,
    total_time: PrettyDuration,
    transfer_time: PrettyDuration,
}

impl Verifier for VerifyTransferTime {
    type Error = InvalidTransferTime;
    type Errors = iter::Once<Self::Error>;

    fn verify(&self, month_file: &MonthFile) -> Result<(), Self::Errors> {
        let total_time = month_file.total_time();
        let succ_transfer = month_file.succ_transfer();
        let expected_working_time = month_file
            .working_time()
            // TODO: why 99?
            .map_or(Duration::from_hours(99), Into::into);

        // transfer_time = total_time - expected_working_time
        // <=> transfer_time + expected_working_time = total_time
        if succ_transfer + expected_working_time != total_time {
            return Err(iter::once(InvalidTransferTime {
                expected_working_time: expected_working_time.into(),
                total_time: total_time.into(),
                transfer_time: succ_transfer.into(),
            }));
        }

        Ok(())
    }
}
