use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::path::PathBuf;

mod options;

#[derive(clap::Parser)]
pub struct Cli {
    // #[arg(short, long, default_value = "./dotfiles")]
    // dotfiles: PathBuf,
    #[command(subcommand)]
    command: Command,

    #[arg(short, long, default_value = "./dotfiles")]
    pub dotfiles: PathBuf,

    #[arg(name = "type", default_value = "UserConfig")]
    config_type: ConfigType,
}

impl Cli {
    pub fn config_dir(&self) -> Result<PathBuf> {
        match self.config_type {
            ConfigType::UserConfig => dirs::config_dir().ok_or(anyhow!("Couldn't find config dir")),
        }
    }
}

#[derive(clap::Subcommand)]
enum Command {
    /// adds a config dir to dotfiles
    Add(options::AddOptions),
    /// removes a config dir from dotfiles or config
    Remove(options::RemoveOptions),
    // Install(),
    // Uninstall(),
    // Backup()
}

// chage and finish this
#[derive(PartialEq, Debug, Clone, clap::ValueEnum)]
enum ConfigType {
    #[value(name = "UserConfig")]
    UserConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = Cli::parse();

    if !options.dotfiles.is_dir() {
        tokio::fs::create_dir(&options.dotfiles)
            .await
            .context("Unable to create dotfiles directory")?;
    }

    match &options.command {
        Command::Add(args) => args.operate(&options).await?,
        Command::Remove(args) => args.operate(&options).await?,
    }

    Ok(())
}
