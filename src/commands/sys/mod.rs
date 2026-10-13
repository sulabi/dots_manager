use std::path::PathBuf;

use anyhow::Result;

use crate::DotsConfig;

mod fonts;
mod packages;
mod services;

#[derive(clap::Subcommand)]
pub enum Command {
    Packages {
        #[command(subcommand)]
        command: packages::Packages,
    },
    Fonts {
        #[command(subcommand)]
        command: fonts::Fonts,
    },
    Services {
        #[command(subcommand)]
        command: services::Services,
    },
}

#[derive(clap::Args)]
pub struct GlobalArgs {}

pub struct CommandArgs {
    dots_directory: PathBuf,
}

impl Command {
    #[allow(unused)]
    pub async fn exec(&self, dots_config: &DotsConfig) -> Result<()> {
        let args = CommandArgs {
            dots_directory: dots_config.dots_directory.clone(),
        };

        match self {
            Self::Packages { command } => command.exec(&args).await,
            Self::Fonts { command } => command.exec(&args).await,
            Self::Services { command } => command.exec(&args).await,
        }
    }
}
