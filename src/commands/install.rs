use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::{
    CommandHandler,
    commands::{PathExt, PathValidate},
};

#[derive(clap::Parser)]
pub struct Install {
    #[arg(short, long, conflicts_with = "all")]
    file: Option<PathBuf>,

    #[arg(short, long)]
    all: bool,
}

impl Install {
    async fn install(&self, file: &Path, handler: &CommandHandler) -> Result<()> {
        let dotfiles = &handler.dotfiles;
        let config_dir = &handler.config_dir;
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

    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        if let Some(f) = &self.file {
            let f = f.ensure_dir()?;

            self.install(f, handler).await?
        }

        if self.all {
            for entry in handler
                .dotfiles
                .read_dir()
                .with_context(|| format!("Failed to read dotfiles dir: {:?}", handler.dotfiles))?
                .flatten()
            {
                let path = entry.path();
                if path.is_dir() {
                    self.install(&path, handler).await?
                }
            }
        }

        Ok(())
    }
}
