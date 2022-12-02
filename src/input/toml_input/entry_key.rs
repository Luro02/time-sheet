use serde::de;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    inner: InnerKey,
}

impl Key {
    #[must_use]
    pub fn day(&self) -> usize {
        let InnerKey::Day(n) = self.inner;
        n
    }
}

impl<'de> de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let number = String::deserialize(deserializer)?
            .parse::<usize>()
            .map_err(de::Error::custom)?;

        if number == 0 || number > 31 {
            return Err(de::Error::custom(format!(
                "Entry key must be between 1 and 31, but was {}",
                number
            )));
        }

        Ok(Self {
            inner: InnerKey::Day(number),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum InnerKey {
    Day(usize),
}
