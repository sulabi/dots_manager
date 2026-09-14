use std::path::PathBuf;

use crate::{CommandArgs, commands::PathValidate};
use anyhow::{Context, Result};

#[derive(clap::Parser)]
pub struct Remove {
    #[arg(short, long)]
    file: PathBuf,

    #[arg(long, default_value_t = false)]
    from_dots: bool,

    #[arg(short, long)]
    all: bool,
}

impl Remove {
    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        let f = &self.file.ensure_dir()?;

        let config_dir = &args.config_dir;
        let config_file = config_dir.join(f);
        config_file.ensure_dir()?.ensure_symlink()?;

        if self.from_dots {
            let absolute = config_file.canonicalize()?;

            tokio::fs::remove_dir_all(absolute)
                .await
                .context("Unable to remove original config")?;
        }

        tokio::fs::remove_dir_all(&config_file)
            .await
            .context("Unable to remove symlink")?;

        Ok(())
    }
}
