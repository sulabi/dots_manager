use std::path::PathBuf;

use anyhow::{Context, Result};

mod add;
mod backup;
mod install;
mod remove;
mod restore;

use crate::DotsConfig;

#[derive(clap::Subcommand)]
pub enum Command {
    /// adds a config file or directory to dotfiles
    Add(add::Add),
    /// removes installed config links, optionally deleting their dotfiles
    Remove(remove::Remove),
    /// symlinks current dotfiles into config
    Install(install::Install),
    /// backs up current dotfiles folder into a tarball
    Backup(backup::Backup),
    /// restores dotfiles from backup tarball but doesn't install
    Restore(restore::Restore),
}

pub struct CommandArgs {
    dot_config: PathBuf,
    dotfiles_dir: PathBuf,
}

impl Command {
    pub async fn exec(&self, config: &DotsConfig) -> Result<()> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;
        let args = CommandArgs {
            dot_config: config.dots_directory.join(".config"),
            dotfiles_dir: config_dir,
        };

        match self {
            Command::Add(command) => command.exec(&args).await,
            Command::Remove(command) => command.exec(&args).await,
            Command::Install(command) => command.exec(&args).await,
            Command::Backup(command) => command.exec(&args).await,
            Command::Restore(command) => command.exec(&args).await,
        }
    }
}
