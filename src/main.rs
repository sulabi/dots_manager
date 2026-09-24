use anyhow::Context;
use appcfg::{Config, ConfigDirectory, Deserialize, Serialize};
use log::{info, warn};
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use anyhow::Result;
use clap::Parser;

mod commands;

use commands::config;
use commands::sys;

#[derive(clap::Subcommand)]
enum Command {
    Config {
        #[command(subcommand)]
        command: config::Command,
    },

    #[command(alias = "sys")]
    System {
        #[command(subcommand)]
        command: sys::Command,
    },

    Init {
        #[arg(long, default_value_os_t = default_directory())]
        directory: PathBuf,
    },
}

fn default_directory() -> PathBuf {
    Config::new(ConfigDirectory::System("dots_manager"))
        .ok()
        .and_then(|config| config.read::<DotsConfig>().ok())
        .map(|dots_config| dots_config.dots_directory)
        .unwrap_or_else(fallback_directory)
}

fn fallback_directory() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let directory = home_dir.join("dotfiles");

    warn!("Initializing at home directory: {}", directory.display());

    directory
}

#[derive(clap::Parser)]
pub struct CommandHandler {
    #[command(subcommand)]
    command: Command,
}

#[derive(Serialize, Deserialize)]
pub struct DotsConfig {
    dots_directory: PathBuf,
}

async fn init(base: &Path) -> Result<()> {
    if !base.exists() {
        fs::create_dir(base)
            .await
            .context("Failed to create base dir")?
    }

    fs::create_dir(base.join(".config"))
        .await
        .context("Failed to write .config folder")?;
    fs::create_dir(base.join("etc"))
        .await
        .context("Failed to write etc folder")?;

    fs::write(base.join("packages.txt"), "")
        .await
        .context("Failed to write packages.txt")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .format_target(false)
        .init();

    let options = CommandHandler::parse();

    let config = Config::new(ConfigDirectory::System("dots_manager"))?;

    let dots_config = config.read::<DotsConfig>()?;

    match options.command {
        Command::Config { command } => command.exec(&dots_config).await,
        Command::System { command } => command.exec(&dots_config).await,

        Command::Init { directory } => {
            info!("Creating init at {}", directory.display());

            init(&directory).await?;

            config.write(&DotsConfig {
                dots_directory: directory,
            })?;

            Ok(())
        }
    }
}
