use std::path::PathBuf;

use crate::CommandHandler;
use anyhow::{Context, Result, anyhow};

#[derive(clap::Parser)]
pub struct Remove {
    #[arg(short, long)]
    file: PathBuf,

    #[arg(long, default_value_t = false)]
    from_dots: bool,
}

impl Remove {
    pub async fn exec(&self, cli: &CommandHandler) -> Result<()> {
        let f = &self.file;

        let config_dir = &cli.config_dir;
        let config_file = config_dir.join(f);

        if !config_file.is_dir() {
            return Err(anyhow!("{:?} is not a set config file", f));
        }
        if !config_file.is_symlink() {
            return Err(anyhow!("{:?} is not a symlink, won't delete", f));
        }

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
