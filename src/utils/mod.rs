use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

use log::trace;
use rust_embed::RustEmbed;
use serde::de::DeserializeOwned;
use serde::ser;

mod iterator;
pub use iterator::*;
mod macros;

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

/// Divides the `number` into `n` equal parts.
///
/// The first returned value is how much each part is allocated and the second is
/// the remainder that can not be distributed equally.
#[must_use]
pub const fn divide_equally(number: usize, n: usize) -> (usize, usize) {
    (number / n, number % n)
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
    fn test_divide_equally() {
        assert_eq!(divide_equally(40, 4), (10, 0));
        assert_eq!(divide_equally(41, 4), (10, 1));
        assert_eq!(divide_equally(42, 4), (10, 2));
        assert_eq!(divide_equally(43, 4), (10, 3));
        assert_eq!(divide_equally(44, 4), (11, 0));
    }
}
