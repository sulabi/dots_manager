use anyhow::Result;
use std::path::PathBuf;

use crate::{CommandHandler, commands::PathValidate};

#[derive(clap::Parser)]
pub struct Backup {
    #[arg(short, long, conflicts_with = "all")]
    file: Option<PathBuf>,
}

#[allow(unused_variables)]
impl Backup {
    pub async fn exec(&self, handler: &CommandHandler) -> Result<()> {
        if let Some(f) = &self.file {
            let dotfiles = &handler.dotfiles;
            let f = f.ensure_dir()?.ensure_child(dotfiles)?;

            todo!()
        }
        Ok(())
    }
}
