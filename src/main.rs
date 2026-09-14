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

        #[command(flatten)]
        args: config::GlobalArgs,
    },

    #[command(alias = "sys")]
    System {
        #[command(subcommand)]
        command: sys::Command,

        #[command(flatten)]
        args: sys::GlobalArgs,
    },
}

#[derive(clap::Parser)]
pub struct CommandHandler {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = CommandHandler::parse();

    match options.command {
        Command::Config { args, command } => command.exec(args).await,
        Command::System { args, command } => command.exec(args).await,
    }
}
