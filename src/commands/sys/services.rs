use anyhow::Result;

use super::CommandArgs;

#[derive(clap::Subcommand)]
pub enum Services {}

impl Services {
    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        Ok(())
    }
}
