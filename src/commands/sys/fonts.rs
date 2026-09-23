use anyhow::Result;

use super::CommandArgs;

#[derive(clap::Subcommand)]
pub enum Fonts {
    List
}

impl Fonts {
    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        Ok(())
    }
}
