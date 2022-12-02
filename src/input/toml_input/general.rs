use serde::Deserialize;

use crate::time::{Date, Month, Year};

#[derive(Debug, Clone, Deserialize)]
pub struct General {
    month: Month,
    year: Year,
    signature: Option<GeneralSignature>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneralSignature {
    date: Date,
}

impl GeneralSignature {
    pub fn date(&self) -> Date {
        self.date
    }
}

impl General {
    pub fn month(&self) -> Month {
        self.month
    }

    pub fn year(&self) -> Year {
        self.year
    }

    pub fn signature(&self) -> Option<&GeneralSignature> {
        self.signature.as_ref()
    }
}
