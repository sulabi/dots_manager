use anyhow::Result;

mod etc;
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
    Etc {
        #[command(subcommand)]
        command: etc::Etc,
    },
}

#[derive(clap::Args)]
pub struct GlobalArgs {
    #[arg(long)]
    hello: bool,
}

pub struct CommandArgs {}

impl Command {
    #[allow(unused)]
    pub async fn exec(&self, args: GlobalArgs) -> Result<()> {
        let args = CommandArgs {};

        match self {
            Self::Packages { command } => command.exec(&args).await,
            Self::Fonts { command } => command.exec(&args).await,
            Self::Etc { command } => command.exec(&args).await,
            Self::Services { command } => command.exec(&args).await,
        }
    }
}
