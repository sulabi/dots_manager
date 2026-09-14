use anyhow::{Context, Result};
use std::path::PathBuf;

use super::CommandArgs;
use crate::commands::{PathExt, PathValidate};

#[derive(clap::Parser)]
pub struct Add {
    #[arg(short, long)]
    file: PathBuf,
}

impl Add {
    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        let f = self.file.ensure_exists()?;
        let dotfiles = &args.dotfiles;
        let config_dir = &args.config_dir;

        let f_name = f.file_name().context("Unable to get file name")?;

        f.ensure_child(config_dir)?;

        // TODO: fix for regular files? idk if works
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
