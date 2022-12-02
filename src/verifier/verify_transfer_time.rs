use std::iter;

use thiserror::Error;

use crate::input::Config;
use crate::time::PrettyDuration;
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

    fn verify(&self, config: &Config) -> Result<(), Self::Errors> {
        let total_time = config.month().total_working_time();
        let transfer_to_next_month = config.month().transfer().next().to_duration();
        let expected_working_duration = config.month().expected_working_duration().to_duration();

        // transfer_time = total_time - expected_working_time
        // <=> transfer_time + expected_working_time = total_time
        if transfer_to_next_month + expected_working_duration != total_time {
            return Err(iter::once(InvalidTransferTime {
                expected_working_time: expected_working_duration.into(),
                total_time: total_time.into(),
                transfer_time: transfer_to_next_month.into(),
            }));
        }

        Ok(())
    }
}
