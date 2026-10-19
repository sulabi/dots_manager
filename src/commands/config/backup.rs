use anyhow::{Context, Result, bail};
use chrono::format::{DelayedFormat, StrftimeItems};
use flate2::{Compression, write::GzEncoder};
use std::path::{Path, PathBuf};

use super::CommandArgs;
use crate::commands::{PathValidate, get_dots};

#[derive(clap::Parser)]
pub struct Backup {
    #[arg(short, long)]
    file: Option<PathBuf>,
}

impl Backup {
    fn get_time() -> DelayedFormat<StrftimeItems<'static>> {
        chrono::Local::now().format("%Y%m%d-%H%M%S-%f")
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        args.dot_config.ensure_dir()?;

        let out_path = self.file.clone().unwrap_or_else(|| {
            args.dot_config
                .join(format!("backup-{}.tar.gz", Self::get_time()))
        });

        let parent = out_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .ensure_dir()?
            .canonicalize()
            .context("Unable to resolve backup destination directory")?;

        let destination = parent.join(out_path.file_name().context("Missing backup file name")?);

        let source = args.dot_config.canonicalize()?;

        if parent.starts_with(&source) && parent != source {
            bail!("Backup destination cannot be inside a managed config directory");
        }

        let mut output = tempfile::NamedTempFile::new_in(&parent)?;
        let temporary_path = output.path().to_path_buf();

        let mut tar =
            tar::Builder::new(GzEncoder::new(output.as_file_mut(), Compression::default()));
        tar.follow_symlinks(false);

        for path in get_dots(&args.dot_config)? {
            let name = path.file_name().context("Missing config file name")?;

            if path.canonicalize().ok().as_ref() == Some(&temporary_path) {
                continue;
            }

            if path.is_dir() && !path.is_symlink() {
                tar.append_dir_all(name, &path)?;
            } else {
                tar.append_path_with_name(&path, name)?;
            }
        }

        tar.into_inner()?.finish()?.sync_all()?;

        output.persist_noclobber(&destination).with_context(|| {
            format!(
                "Failed to save backup to {} (destination must not exist)",
                destination.display()
            )
        })?;

        Ok(())
    }
}
