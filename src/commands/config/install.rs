use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::commands::{PathExt, PathValidate, get_dots};

use super::CommandArgs;

#[derive(clap::Parser)]
pub struct Install {
    #[arg(short, long)]
    file: Option<PathBuf>,
}

impl Install {
    async fn install(&self, file: &Path, args: &CommandArgs) -> Result<()> {
        let dot_config = args.dot_config.ensure_dir()?;
        let config_dir = args.dotfiles_dir.ensure_dir()?;
        let file = file.ensure_exists()?.ensure_child(dot_config)?;

        let target = file.relative_path(config_dir)?;
        let name = file.file_name().context("Failed to get file name")?;
        let link = config_dir.join(name);

        if link.symlink_metadata().is_ok() {
            if link.is_symlink() && link.link_matches(file)? {
                return Ok(());
            }

            bail!(
                "{} already exists and does not link to {}",
                link.display(),
                file.display()
            );
        }

        tokio::fs::symlink(&target, &link).await.with_context(|| {
            format!(
                "Failed to create symlink {} to {}",
                link.display(),
                target.display()
            )
        })?;

        Ok(())
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        if let Some(file) = &self.file {
            let file = file.selected_path(&args.dot_config)?;

            return self.install(&file, args).await;
        }

        for path in get_dots(&args.dot_config)? {
            // TODO: make a better terminal output
            println!(
                "installing -> {:?}",
                path.file_name().context("Failed to get file name")?
            );

            self.install(&path, args).await?;
        }

        Ok(())
    }
}
