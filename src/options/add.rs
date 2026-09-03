use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::Cli;

#[derive(clap::Parser)]
pub struct AddOptions {
    #[arg(short, long)]
    file: PathBuf,
}

impl AddOptions {
    pub async fn operate(&self, cli: &Cli) -> Result<()> {
        let f = &self.file;
        let dotfiles = &cli.dotfiles;
        let config_dir = &cli.config_dir()?;

        if !f.is_dir() {
            anyhow::bail!("{:?} is not a valid dir", f);
        }

        let f_name = f.file_name().context("Unable to get file name")?;
        if f.parent().context("Unable to get file parent")? != config_dir {
            anyhow::bail!(
                "{} is not a child of config directory",
                f_name.to_string_lossy()
            );
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

        tokio::fs::symlink(target, config_dir.join(f_name))
            .await
            .context("Symlink err")?;

        Ok(())
    }
}
