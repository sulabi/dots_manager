use std::path::PathBuf;

use anyhow::{Context, Result};

use appcfg::{Config, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DotsConfig {
    pub dotfiles: PathBuf,
}

#[derive(clap::Args)]
pub struct Init {
    #[arg(short, long)]
    dotfiles: Option<PathBuf>,
}

impl Init {
    pub fn exec(&self, config: &Config) -> Result<()> {
        let dotfiles = match &self.dotfiles {
            Some(d) => d.clone(),
            None => std::env::current_dir().context("Unable to get current dir")?,
        };

        config
            .write(&DotsConfig { dotfiles })
            .context("Failed to save dotfiles path to config")
    }
}
