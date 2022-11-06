use serde::Deserialize;

use crate::time::{Month, Year};

#[derive(Debug, Clone, Deserialize)]
pub struct General {
    month: Month,
    year: Year,
}

impl General {
    pub fn month(&self) -> Month {
        self.month
    }

    pub fn year(&self) -> Year {
        self.year
    }
}
