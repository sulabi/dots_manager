use anyhow::Result;

use super::CommandArgs;

#[derive(clap::Subcommand)]
pub enum Etc {}

impl Etc {
    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        Ok(())
    }
}
