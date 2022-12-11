use std::str::FromStr;

use derive_more::Display;
use serde::{de, ser, Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Display)]
pub enum WorkingArea {
    #[display(fmt = "ub")]
    Universitary,
    #[display(fmt = "gf")]
    LargeScaleResearchSector,
}

impl FromStr for WorkingArea {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "ub" => Ok(Self::Universitary),
            "gf" => Ok(Self::LargeScaleResearchSector),
            _ => Err(anyhow::anyhow!("Invalid working area: {}", string)),
        }
    }
}

impl<'de> Deserialize<'de> for WorkingArea {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for WorkingArea {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
