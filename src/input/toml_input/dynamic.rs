use serde::Deserialize;

use crate::time::WorkingDuration;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum DynamicEntryInput {
    Flex { flex: usize },
    Fixed { duration: WorkingDuration },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DynamicEntry {
    #[serde(flatten)]
    input: DynamicEntryInput,
}

impl DynamicEntry {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    use crate::working_duration;
    use pretty_assertions::assert_eq;

    #[derive(Debug, Clone, Deserialize)]
    struct EntrySections {
        pub entry: HashMap<String, DynamicEntry>,
    }

    macro_rules! map {
        ( $( $key:expr => $value:expr ),+ $(,)? ) => {
            {
                let mut _map = ::std::collections::HashMap::new();

                $(
                    _map.insert($key, $value);
                )+

                _map
            }
        };
    }

    #[test]
    fn test_deserialize_flex() {
        let input = concat!("[entry.\"first example\"]\n", "flex = 1\n",);

        let sections: EntrySections = toml::from_str(input).unwrap();

        assert_eq!(
            sections.entry,
            map! {
                "first example".to_string() => DynamicEntry {
                    input: DynamicEntryInput::Flex { flex: 1 },
                },
            }
        );
    }

    #[test]
    fn test_deserialize_fixed() {
        let input = concat!("[entry.\"another example\"]\n", "duration = \"41:23\"\n",);

        let sections: EntrySections = toml::from_str(input).unwrap();

        assert_eq!(
            sections.entry,
            map! {
                "another example".to_string() => DynamicEntry {
                    input: DynamicEntryInput::Fixed { duration: working_duration!(41:23) },
                },
            }
        );
    }
}
