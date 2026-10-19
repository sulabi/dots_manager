use anyhow::Result;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

mod path;

pub use path::{PathExt, PathValidate};

pub mod config;
pub mod sys;

pub fn list_dir(path: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path()))
}

pub fn get_dots(dots: &Path) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(list_dir(dots)?.filter(move |entry| entry.extension() != Some(OsStr::new("gz"))))
}
