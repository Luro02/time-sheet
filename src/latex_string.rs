use std::ops::Deref;
use std::str::FromStr;

use derive_more::Display;
use serde::{de, Deserialize, Serialize};

#[derive(Debug, Clone, Display, PartialEq, Serialize)]
#[display("{}", _0)]
pub struct LatexString(String);

impl FromStr for LatexString {
    type Err = !;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(LatexString(v_latexescape::escape(value).to_string()))
    }
}

impl Deref for LatexString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for LatexString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
