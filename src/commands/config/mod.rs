use std::path::PathBuf;

use anyhow::{Context, Result};

mod add;
mod backup;
mod init;
mod install;
mod remove;
mod restore;

use appcfg::{Config, ConfigDirectory};
pub use init::*;

#[derive(clap::Args)]
pub struct GlobalArgs {
    #[arg(long, global = true, default_value_os_t = dotfiles_default())]
    dotfiles: PathBuf,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// adds a config dir to dotfiles
    Add(add::Add),
    /// removes a config dir from dotfiles or config
    Remove(remove::Remove),
    /// symlinks current dotfiles folders into config
    Install(install::Install),
    /// backs up current dotfiles folder into a tarball
    Backup(backup::Backup),
    /// restores dotfiles from backup tarball but doesn't install
    Restore(restore::Restore),

    /// sets the current directory as a saved dotfiles path
    Init(init::Init),
}

pub struct CommandArgs {
    dotfiles: PathBuf,
    config_dir: PathBuf,
}

impl Command {
    pub async fn exec(&self, args: GlobalArgs) -> Result<()> {
        let config = Config::new(ConfigDirectory::System("dots_manager"))?;

        if let Command::Init(args) = &self {
            return args.exec(&config);
        }

        let config_dir = dirs::config_dir().context("Failed to get config directory")?;
        let args = CommandArgs {
            dotfiles: args.dotfiles,
            config_dir,
        };

        match self {
            Command::Add(command) => command.exec(&args).await,
            Command::Remove(command) => command.exec(&args).await,
            Command::Install(command) => command.exec(&args).await,
            Command::Backup(command) => command.exec(&args).await,
            Command::Restore(command) => command.exec(&args).await,

            Command::Init(_) => unreachable!(),
        }
    }
}

pub fn dotfiles_default() -> PathBuf {
    let config = match Config::new(ConfigDirectory::System("dots_manager")) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to get config, make sure to run init: {e}");
            std::process::exit(1);
        }
    };

    match config.read::<DotsConfig>() {
        Ok(config) => config.dotfiles,
        Err(e) => {
            eprintln!("Failed to read config, make sure to run init: {e}");
            std::process::exit(1);
        }
    }
}
