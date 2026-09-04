use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;

use crate::CommandHandler;

#[derive(clap::Parser)]
pub struct Add {
    #[arg(short, long)]
    file: PathBuf,
}

impl Add {
    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        let f = &self.file;
        let dotfiles = &handler.dotfiles;
        let config_dir = &handler.config_dir;

        if !f.is_dir() {
            return Err(anyhow!("{:?} is not a valid dir", f));
        }

        let f_name = f.file_name().context("Unable to get file name")?;
        if !f.starts_with(config_dir) {
            return Err(anyhow!(
                "{} is not a child of config directory",
                f_name.to_string_lossy()
            ));
        }

        let link = dotfiles.join(f_name);
        tokio::fs::rename(f, &link).await.with_context(|| {
            format!(
                "Unable to move file {} into {:?}",
                f_name.to_string_lossy(),
                dotfiles
            )
        })?;

        let target = pathdiff::diff_paths(link.canonicalize()?, config_dir)
            .context("unable to get relative path")?;

        let link = config_dir.join(f_name);

        tokio::fs::symlink(target, link)
            .await
            .context("Symlink err")?;

        Ok(())
    }
}
