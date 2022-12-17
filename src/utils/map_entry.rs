use std::marker::PhantomData;

use serde::de;

pub trait MapEntry<'de> {
    type Key: de::Deserialize<'de>;
    type Value: de::Deserialize<'de>;

    #[must_use]
    fn new(key: Self::Key, value: Self::Value) -> Self;
}

struct MapEntryVisitor<T> {
    marker: PhantomData<T>,
}

impl<T> Default for MapEntryVisitor<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<'de, T: MapEntry<'de>> de::Visitor<'de> for MapEntryVisitor<T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut items = Vec::with_capacity(map.size_hint().unwrap_or(0));
        while let Some((key, value)) = map.next_entry::<T::Key, T::Value>()? {
            items.push(T::new(key, value));
        }

        Ok(items)
    }
}

pub fn deserialize_map_entry<'de, D: de::Deserializer<'de>, E: MapEntry<'de>>(
    deserializer: D,
) -> Result<Vec<E>, D::Error> {
    deserializer.deserialize_map(MapEntryVisitor::default())
}
