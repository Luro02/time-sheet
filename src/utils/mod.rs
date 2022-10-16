use std::fs::{self, File};
use std::io;
use std::path::Path;

use log::trace;
use rust_embed::RustEmbed;
use serde::ser;

#[derive(RustEmbed)]
#[folder = "resources/"]
pub struct Resources;

pub fn round_serialize<S>(x: &f32, s: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    s.serialize_f32(*x)
}

#[must_use]
pub fn overflowing_add(base: u64, to_add: u64, limit: u64) -> u64 {
    if base + to_add >= limit {
        base + to_add - limit
    } else {
        base + to_add
    }
}

pub fn open(path: impl AsRef<Path>) -> io::Result<File> {
    trace!("opening: {}", path.as_ref().display());
    File::open(path)
}

pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    trace!("reading from: {}", path.as_ref().display());
    fs::read_to_string(path)
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    trace!("writing to: {}", path.as_ref().display());
    fs::write(path, contents)
}
