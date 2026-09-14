use anyhow::{Context, Result};
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
        let dotfiles = &args.dotfiles;
        let config_dir = &args.config_dir;
        let f = file.ensure_dir()?.ensure_child(dotfiles)?;

        let target = f.relative_path(config_dir)?;
        let f_name = f.file_name().context("Failed to get file name")?;

        let link = config_dir.join(f_name);

        if !link.is_symlink() {
            tokio::fs::symlink(&target, &link)
                .await
                .with_context(|| format!("Symlink error on {:?} to {:?}", target, link))?;
        }

        Ok(())
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        if let Some(f) = &self.file {
            let f = f.ensure_dir()?;

            self.install(f, args).await?
        }

        let files = get_dots(&args.dotfiles)?;

        for path in files {
            if path.is_dir() {
                // TODO: make a better terminal output
                println!(
                    "installing -> {:?}",
                    path.file_name().context("Failed to get file name")?
                );

                self.install(&path, args).await?
            }
        }

        Ok(())
    }
}
