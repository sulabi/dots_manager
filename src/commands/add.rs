use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::{
    CommandHandler,
    commands::{PathExt, PathValidate},
};

#[derive(clap::Parser)]
pub struct Add {
    #[arg(short, long)]
    file: PathBuf,
}

impl Add {
    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        let f = &self.file.ensure_dir()?;
        let dotfiles = &handler.dotfiles;
        let config_dir = &handler.config_dir;

        let f_name = f.file_name().context("Unable to get file name")?;

        f.ensure_child(config_dir)?;

        let link = dotfiles.join(f_name);
        f.move_dir(link.as_path()).await?;

        let target = link.relative_path(config_dir)?;
        let link = config_dir.join(f_name);

        tokio::fs::symlink(target, link)
            .await
            .context("Symlink err")?;

        Ok(())
    }
}
