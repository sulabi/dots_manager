use std::path::{Path, PathBuf};

use super::CommandArgs;
use crate::commands::{PathExt, PathValidate};
use anyhow::{Context, Result, bail};

#[derive(clap::Parser)]
pub struct Remove {
    #[arg(short, long, required_unless_present = "all", conflicts_with = "all")]
    file: Option<PathBuf>,

    /// Also delete the corresponding file or directory from dotfiles
    #[arg(long, default_value_t = false)]
    from_dots: bool,

    /// Remove all installed links belonging to this dotfiles directory
    #[arg(short, long)]
    all: bool,
}

impl Remove {
    fn managed_file(&self, link: &Path, args: &CommandArgs) -> Result<PathBuf> {
        link.ensure_symlink()?;

        let name = link.file_name().context("Failed to get file name")?;
        let file = args.dot_config.join(name);

        if !link.link_matches(&file)? {
            bail!("{} does not link to {}", link.display(), file.display());
        }

        Ok(file)
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        args.dot_config.ensure_dir()?;
        args.dotfiles_dir.ensure_dir()?;

        let mut links = Vec::new();

        if self.all {
            for entry in std::fs::read_dir(&args.dotfiles_dir)? {
                let path = entry?.path();

                if !path.is_symlink() {
                    continue;
                }

                let name = path.file_name().context("Failed to get file name")?;
                let file = args.dot_config.join(name);

                if path.link_matches(&file)? {
                    links.push(path);
                }
            }
        } else {
            let file = self.file.as_ref().context("Specify --file or --all")?;

            links.push(file.selected_path(&args.dotfiles_dir)?);
        }

        // Validate every selected link before removing anything.
        let files = links
            .iter()
            .map(|link| self.managed_file(link, args))
            .collect::<Result<Vec<_>>>()?;

        for (link, file) in links.iter().zip(files) {
            if self.from_dots {
                match tokio::fs::symlink_metadata(&file).await {
                    Ok(metadata) if metadata.is_dir() => {
                        tokio::fs::remove_dir_all(&file)
                            .await
                            .with_context(|| format!("Unable to remove {}", file.display()))?;
                    }
                    Ok(_) => {
                        tokio::fs::remove_file(&file)
                            .await
                            .with_context(|| format!("Unable to remove {}", file.display()))?;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(error.into()),
                }
            }

            tokio::fs::remove_file(link)
                .await
                .with_context(|| format!("Unable to remove symlink {}", link.display()))?;
        }

        Ok(())
    }
}
