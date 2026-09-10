use anyhow::{Context, Result};
use chrono::format::{DelayedFormat, StrftimeItems};
use flate2::{Compression, write::GzEncoder};
use std::fs::File;
use std::path::PathBuf;

use crate::commands::get_dots;
use crate::{CommandHandler, commands::PathValidate};

#[derive(clap::Parser)]
pub struct Backup {
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[allow(unused_variables)]
impl Backup {
    fn get_time() -> DelayedFormat<StrftimeItems<'static>> {
        chrono::Local::now().format("%d%m%Y-%H%M%S")
    }

    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        let dotfiles = &handler.dotfiles;

        let out_path = if let Some(f) = &self.file {
            f.ensure_dir()?.ensure_child(dotfiles)?
        } else {
            &dotfiles.join(format!("backup-{}.tar.gz", Backup::get_time()))
        };

        let tar_gz = File::create(out_path)
            .with_context(|| format!("Failed to create backup file: {}", out_path.display()))?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);
        tar.follow_symlinks(false);

        for path in get_dots(dotfiles)? {
            let file_name = path
                .file_name()
                .with_context(|| format!("Couldn't get file name: {:?}", path))?;

            if path.is_dir() {
                tar.append_dir_all(file_name, &path)?;
            } else {
                tar.append_path_with_name(&path, file_name)?;
            }
        }

        tar.finish()?;

        Ok(())
    }
}
