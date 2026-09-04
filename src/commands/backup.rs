use anyhow::Result;
use flate2::{Compression, write::GzEncoder};
use std::fs::File;
use std::path::PathBuf;

use crate::{CommandHandler, commands::PathValidate};

#[derive(clap::Parser)]
pub struct Backup {
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[allow(unused_variables)]
impl Backup {
    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        if let Some(f) = &self.file {
            let dotfiles = &handler.dotfiles;
            let f = f.ensure_dir()?.ensure_child(dotfiles)?;

            let tar_gz = File::create(f.with_added_extension("bak.tar.gz"))?;
            let enc = GzEncoder::new(tar_gz, Compression::default());
            let mut tar = tar::Builder::new(enc);
            tar.follow_symlinks(false);
            tar.append_dir_all(dotfiles, f)?;
            tar.finish()?;
        }
        Ok(())
    }
}
