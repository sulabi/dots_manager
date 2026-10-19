use anyhow::{Context, Result, bail};
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
        let dot_config = args.dot_config.ensure_dir()?;
        let config_dir = args.dotfiles_dir.ensure_dir()?;
        let file = self.file.selected_path(config_dir)?;

        file.ensure_exists()?.ensure_child(config_dir)?;

        if file.is_symlink() {
            bail!("{} is already a symlink", file.display());
        }

        let name = file.file_name().context("Unable to get file name")?;
        let destination = dot_config.join(name);

        if destination.symlink_metadata().is_ok() {
            bail!("{} already exists in dotfiles", destination.display());
        }

        let target = dot_config.relative_path(config_dir)?.join(name);

        file.move_dir(&destination).await?;

        if let Err(error) = tokio::fs::symlink(&target, &file).await {
            destination
                .move_dir(&file)
                .await
                .with_context(|| format!("Failed to create symlink: {error}; failed to return config to {}. Config remains at {}", file.display(), destination.display()))?;

            return Err(error).context("Failed to create symlink; original config restored");
        }

        Ok(())
    }
}
