use anyhow::Context;
use clap::CommandFactory;
use configfs::{Config, ConfigDirectory, Deserialize, Serialize};
use log::info;
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

    /// Initialise the dotfiles directory, and the config file if not already created
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

    home_dir.join("dotfiles")
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

    fs::write(base.join("fonts.txt"), "")
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

    let dots_config = match config.read::<DotsConfig>() {
        Ok(config) if config.dots_directory.exists() => config,
        _ => {
            eprintln!("No config or dotfiles directory found.\n");

            if !matches!(&options.command, Command::Init { .. }) {
                let mut cmd = CommandHandler::command();
                if let Some(init_cmd) = cmd.find_subcommand_mut("init") {
                    init_cmd.print_help()?;
                }
                anyhow::bail!("run `init` first");
            }

            DotsConfig {
                dots_directory: PathBuf::new(),
            }
        }
    };

    match options.command {
        Command::Config { command } => command.exec(&dots_config).await,
        Command::System { command } => command.exec(&dots_config).await,

        Command::Init { directory } => {
            info!("Creating init at {}...", directory.display());

            init(&directory).await?;

            config.write(&DotsConfig {
                dots_directory: directory,
            })?;
            info!("Done! You may run the other commands now!");
            Ok(())
        }
    }
}
