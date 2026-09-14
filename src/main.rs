use anyhow::{Context, Result};
use appcfg::{Config, ConfigDirectory};
use clap::{Parser, ValueEnum};
use std::{fmt::Display, path::PathBuf};

use crate::commands::DotsConfig;

mod commands;

#[derive(clap::Parser)]
pub struct CommandHandler {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long)]
    dotfiles: Option<PathBuf>,

    #[arg(name = "type", default_value = "UserConfig")]
    config_type: ConfigType,
}

impl CommandHandler {
    fn resolve_dotfiles(&self, config: &Config) -> Result<PathBuf> {
        config
            .read()
            .map(|DotsConfig { dotfiles }| dotfiles)
            .context("No dotfiles path set")
    }
}

#[derive(clap::Subcommand)]
enum Command {
    /// adds a config dir to dotfiles
    Add(commands::Add),
    /// removes a config dir from dotfiles or config
    Remove(commands::Remove),
    /// symlinks current dotfiles folders into config
    Install(commands::Install),
    /// backs up current dotfiles folder into a tarball
    Backup(commands::Backup),
    /// restores dotfiles from backup tarball but doesn't install
    Restore(commands::Restore),

    Init(commands::Init),
}

// chage and finish this
#[derive(PartialEq, Debug, Clone, clap::ValueEnum)]
enum ConfigType {
    #[value(name = "UserConfig")]
    UserConfig,
}

impl Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .map(|pv| pv.get_name().to_string())
            .unwrap_or_default()
            .fmt(f)
    }
}

struct CommandArgs {
    dotfiles: PathBuf,
    config_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = CommandHandler::parse();
    let config = Config::new(ConfigDirectory::System("dots_manager"))?;

    if let Command::Init(args) = &options.command {
        return args.exec(&config);
    }

    let dotfiles = options.resolve_dotfiles(&config)?;

    if !dotfiles.is_dir() {
        tokio::fs::create_dir_all(&dotfiles)
            .await
            .context("Unable to create dotfiles directory")?;
    }

    let command_args = CommandArgs {
        dotfiles,
        config_dir: match options.config_type {
            ConfigType::UserConfig => dirs::config_dir().context("Could not get config dir")?,
        },
    };

    match &options.command {
        Command::Add(args) => args.exec(&command_args).await,
        Command::Remove(args) => args.exec(&command_args).await,
        Command::Install(args) => args.exec(&command_args).await,
        Command::Backup(args) => args.exec(&command_args).await,
        Command::Restore(args) => args.exec(&command_args).await,

        Command::Init(_) => unreachable!(),
    }
}
