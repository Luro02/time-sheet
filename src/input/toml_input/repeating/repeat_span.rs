use std::str::FromStr;

use serde::Deserialize;

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(try_from = "String")]
#[serde(untagged)]
pub enum RepeatSpan {
    Day,
    Week,
    Month,
    Year,
}

impl FromStr for RepeatSpan {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "day" | "days" | "daily" => Ok(Self::Day),
            "week" | "weeks" | "weekly" => Ok(Self::Week),
            "month" | "months" | "monthly" => Ok(Self::Month),
            "year" | "years" | "yearly" => Ok(Self::Year),
            _ => Err(anyhow::anyhow!("Invalid repeat span: {}", s)),
        }
    }
}

impl TryFrom<String> for RepeatSpan {
    type Error = <Self as FromStr>::Err;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}
