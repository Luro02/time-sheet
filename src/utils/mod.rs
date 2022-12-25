use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

use log::trace;
use rust_embed::RustEmbed;
use serde::de::DeserializeOwned;
use serde::ser;

use crate::iter_const;

mod iterator;
mod macros;
mod map_entry;

pub use iterator::*;
pub use map_entry::*;

#[derive(RustEmbed)]
#[folder = "resources/"]
pub struct Resources;

pub fn round_serialize<S>(x: &f32, s: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    s.serialize_f32(*x)
}

pub fn toml_from_reader<R, T>(reader: R) -> anyhow::Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut reader = BufReader::new(reader);
    let mut date = String::with_capacity(1024 * 1024);
    reader.read_to_string(&mut date)?;
    Ok(toml::from_str(&date)?)
}

pub mod serde_toml_local_date {
    use core::fmt;

    use toml::value::{Date, Datetime};

    use serde::de::{self, Deserialize};
    use serde::ser::{self, Serialize};

    // NOTE: `toml::value::Datetime` is used, because
    // `toml::value::Date` does not implement `Deserialize`

    pub fn serialize<S, T>(date: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: Into<Date> + Clone,
    {
        Datetime {
            date: Some(date.clone().into()),
            time: None,
            offset: None,
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: de::Deserializer<'de>,
        T: TryFrom<Date>,
        T::Error: fmt::Display,
    {
        let date = Datetime::deserialize(deserializer)?
            .date
            .ok_or_else(|| de::Error::custom("expected a date"))?;

        T::try_from(date).map_err(de::Error::custom)
    }
}

// TODO: what about multiple overflow? or when base + to_add overflows?
#[must_use]
pub fn overflowing_add(base: u64, to_add: u64, limit: u64) -> u64 {
    if base + to_add > limit {
        base + to_add - limit
    } else {
        base + to_add
    }
}

pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    trace!("reading from: {}", path.as_ref().display());
    fs::read_to_string(path)
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    trace!("writing to: {}", path.as_ref().display());
    fs::write(path, contents)
}

pub trait PathExt {
    #[must_use]
    fn has_extension<E>(&self, extension: E) -> bool
    where
        for<'a> &'a OsStr: PartialEq<E>;
}

impl PathExt for Path {
    fn has_extension<E>(&self, extension: E) -> bool
    where
        for<'a> &'a OsStr: PartialEq<E>,
    {
        self.extension().map_or(false, |ext| ext == extension)
    }
}

pub trait ArrayExt<T, const N: usize> {
    #[must_use]
    fn init_with(f: impl FnMut(usize) -> T) -> [T; N];

    #[must_use]
    fn init(value: T) -> [T; N]
    where
        T: Clone,
    {
        Self::init_with(|_| value.clone())
    }
}

impl<T, const N: usize> ArrayExt<T, N> for [T; N] {
    fn init_with(mut f: impl FnMut(usize) -> T) -> [T; N] {
        let mut i = 0;
        [(); N].map(|_| {
            let value = f(i);
            i += 1;
            value
        })
    }
}

#[must_use]
const fn sum_array<const N: usize>(array: [usize; N]) -> usize {
    let mut sum = 0;
    let mut i = 0;
    while i < N {
        sum += array[i];
        i += 1;
    }
    sum
}

/// Divides the `numerator` into `N` parts, sized proportionally to the
/// `proportion` values.
///
/// # Examples
///
/// ```ignore
/// # use time_sheet::utils::divide_proportionally;
/// #
/// // Suppose you have 10 hours and proportion is [1, 2, 3, 4]
/// // sum([1, 2, 3, 4]) = 10
///
/// // Then [1/10, 2/10, 3/10, 4/10] is how much each part gets of the 10 hours
/// // -> [1/10 * 10, 2/10 * 10, 3/10 * 10, 4/10 * 10] = [1, 2, 3, 4]
/// let (result, remainder) = divide_proportionally(10, [1, 2, 3, 4]);
///
/// assert_eq!(remainder, 0);
/// assert_eq!(result, [1, 2, 3, 4]);
/// ```
pub const fn divide_proportionally<const N: usize>(
    numerator: usize,
    proportion: [usize; N],
) -> ([usize; N], usize) {
    let total = sum_array(proportion);

    let mut result = [0; N];
    let mut remainder = numerator;

    iter_const!(for i in 0,..N => {
        result[i] = (numerator * proportion[i]) / total;
        remainder -= result[i];
    });

    (result, remainder)
}

pub trait StrExt {
    fn split_exact<const N: usize>(&self, pat: &str) -> [Option<&str>; N];
}

impl StrExt for str {
    fn split_exact<const N: usize>(&self, pat: &str) -> [Option<&str>; N] {
        let mut split = self.splitn(N, pat);
        [(); N].map(|_| split.next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_divide_proportionally() {
        assert_eq!(divide_proportionally(10, [1, 2, 3, 4]), ([1, 2, 3, 4], 0));
        assert_eq!(divide_proportionally(11, [1, 2, 3, 4]), ([1, 2, 3, 4], 1));
        assert_eq!(
            divide_proportionally(2460, [1920, 2880, 2880, 2880, 1440, 0,]),
            ([393, 590, 590, 590, 295, 0], 2)
        );
    }
}
