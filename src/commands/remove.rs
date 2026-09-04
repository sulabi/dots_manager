use std::path::PathBuf;

use crate::{CommandHandler, commands::PathValidate};
use anyhow::{Context, Result};

#[derive(clap::Parser)]
pub struct Remove {
    #[arg(short, long)]
    file: PathBuf,

    #[arg(long, default_value_t = false)]
    from_dots: bool,
}

impl Remove {
    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        let f = &self.file.ensure_dir()?;

        let config_dir = &handler.config_dir;
        let config_file = config_dir.join(f);
        config_file.ensure_dir()?.ensure_symlink()?;

        tokio::fs::remove_dir_all(&config_file)
            .await
            .context("Unable to remove symlink")?;

        if self.from_dots {
            let link = config_file.read_link().context("Failed to read link")?;
            tokio::fs::remove_dir_all(link)
                .await
                .context("Unable to remove original config")?;
        }

        Ok(())
    }
}
