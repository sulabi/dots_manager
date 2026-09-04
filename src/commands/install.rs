use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;

use crate::CommandHandler;

#[derive(clap::Parser)]
pub struct Install {
    #[arg(short, long, conflicts_with = "all")]
    file: Option<PathBuf>,

    #[arg(short, long)]
    all: bool,
}

impl Install {
    async fn install(&self, file: &PathBuf, handler: &CommandHandler) -> Result<()> {
        let f = file;
        let dotfiles = &handler.dotfiles;
        let config_dir = &handler.config_dir;

        if !f.is_dir() {
            return Err(anyhow!("{:?} is not a valid dir", f));
        }

        if !f.starts_with(dotfiles) {
            println!("{:?} parent = {:?}", f, f.parent());
            return Err(anyhow!("{:?} is not in a dotfiles folder", f));
        }

        let target = pathdiff::diff_paths(f.canonicalize()?, config_dir)
            .context("unable to get relative path")?;
        let f_name = f.file_name().context("Failed to get file name")?;

        let link = config_dir.join(f_name);

        if !link.is_symlink() {
            tokio::fs::symlink(target, link)
                .await
                .context("Symlink err")?;
        }

        Ok(())
    }

    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        if let Some(f) = &self.file {
            if !f.is_dir() {
                return Err(anyhow!("{:?} is not a valid dir", f));
            }

            self.install(f, handler).await?
        }

        if self.all {
            for entry in handler
                .dotfiles
                .read_dir()
                .context("Failed to read dotfiles dir:")?
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
