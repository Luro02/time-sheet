use crate::input::json_input::MonthFile;

mod verifier;
mod verify_not_sunday;
mod verify_time;
mod verify_transfer_time;

pub use verifier::Verifier;
pub use verify_not_sunday::*;
pub use verify_time::*;
pub use verify_transfer_time::*;

pub struct DefaultVerifier;

impl Verifier for DefaultVerifier {
    type Error = anyhow::Error;
    type Errors = Vec<Self::Error>;

    fn verify(&self, month_file: &MonthFile) -> Result<(), Self::Errors> {
        VerifyNotSunday
            .verify(month_file)
            .map_err(|errors| errors.into_iter().map(Into::into).collect::<Self::Errors>())?;

        // TODO: this is broken
        /*
        VerifyTransferTime
            .verify(month_file)
            .map_err(|errors| errors.into_iter().map(Into::into).collect::<Self::Errors>())?;
        */

        Ok(())
    }
}

impl Verifier for () {
    type Error = !;
    type Errors = [Self::Error; 1];

    fn verify(&self, _month_file: &MonthFile) -> Result<(), Self::Errors> {
        Ok(())
    }
}

/*
impl<A, B> Verifier for Either<A, B>
where
    A: Verifier,
    B: Verifier,
    A::Errors: IntoIterator<Item = A::Error>,
    B::Errors: IntoIterator<Item = B::Error>,
{
    type Error = Either<A::Error, B::Error>;
    type Errors =
        Either<<A::Errors as IntoIterator>::IntoIter, <B::Errors as IntoIterator>::IntoIter>;

    fn verify(&self, month_file: &MonthFile) -> Result<(), Self::Errors> {
        match self {
            Self::Left(l) => l
                .verify(month_file)
                .map_err(|e| Either::Left(e.into_iter())),
            Self::Right(r) => r
                .verify(month_file)
                .map_err(|e| Either::Right(e.into_iter())),
        }
    }
}*/
