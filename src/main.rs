use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::{fmt::Display, path::PathBuf};

mod commands;

#[derive(clap::Parser)]
pub struct CommandHandler {
    // #[arg(short, long, default_value = "./dotfiles")]
    // dotfiles: PathBuf,
    #[command(subcommand)]
    command: Command,

    #[arg(short, long, default_value = "./dotfiles")]
    pub dotfiles: PathBuf,

    #[arg(name = "type", default_value = "UserConfig")]
    config_type: ConfigType,

    #[arg(skip)]
    config_dir: PathBuf,
}

impl CommandHandler {
    fn init() -> Result<Self> {
        let mut handler = Self::parse();

        handler.config_dir = match handler.config_type {
            ConfigType::UserConfig => dirs::config_dir().context("Could not get config dir")?,
        };

        Ok(handler)
    }
}

#[derive(clap::Subcommand)]
enum Command {
    /// adds a config dir to dotfiles
    Add(commands::Add),
    /// removes a config dir from dotfiles or config
    Remove(commands::Remove),
    Install(commands::Install),
    // Uninstall(),
    // Backup()
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

#[tokio::main]
async fn main() -> Result<()> {
    let options = CommandHandler::init()?;

    if !options.dotfiles.is_dir() {
        tokio::fs::create_dir(&options.dotfiles)
            .await
            .context("Unable to create dotfiles directory")?;
    }

    match &options.command {
        Command::Add(args) => args.exec(&options).await?,
        Command::Remove(args) => args.exec(&options).await?,
        Command::Install(args) => args.exec(&options).await?,
    }

    Ok(())
}
