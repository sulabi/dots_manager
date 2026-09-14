use anyhow::Result;

#[derive(clap::Subcommand)]
pub enum Command {}

#[derive(clap::Args)]
pub struct GlobalArgs {
    hello: bool,
}

impl Command {
    #[allow(unused)]
    pub async fn exec(&self, args: GlobalArgs) -> Result<()> {
        Ok(())
    }
}
