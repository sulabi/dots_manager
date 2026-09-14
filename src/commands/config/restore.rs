use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::{ffi::OsStr, fs::File, path::PathBuf};
use tar::Archive;

use super::CommandArgs;
use crate::commands::{PathValidate, get_dots, list_dir};

#[derive(clap::Parser)]
pub struct Restore {
    #[arg(short, long)]
    from_file: Option<PathBuf>,
}

impl Restore {
    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        let dotfiles = &args.dotfiles;

        dotfiles.ensure_dir()?;

        let restore_file = match self.from_file.clone() {
            Some(f) => f,
            None => list_dir(dotfiles)?
                .filter(|path| path.extension() == Some(OsStr::new("gz")))
                .max_by_key(|path| {
                    path.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                })
                .context("No backup file found")?,
        };

        for path in get_dots(dotfiles)? {
            if path.is_dir() && !path.is_symlink() {
                tokio::fs::remove_dir_all(&path)
                    .await
                    .with_context(|| format!("Failed to remove {:?}", path))?;
            } else {
                tokio::fs::remove_file(&path)
                    .await
                    .with_context(|| format!("Failed to remove {:?}", path))?;
            }
        }

        let tar_gz = File::open(&restore_file)
            .with_context(|| format!("Failed to open restore file: {:?}", restore_file))?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        archive
            .unpack(dotfiles)
            .with_context(|| format!("failed to unpack archive {:?}", restore_file))?;

        Ok(())
    }
}
