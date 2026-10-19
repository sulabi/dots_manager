use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
};
use tar::Archive;

use super::CommandArgs;
use crate::commands::{PathExt, PathValidate};

#[derive(clap::Parser)]
pub struct Restore {
    #[arg(short, long)]
    from_file: Option<PathBuf>,
}

impl Restore {
    fn contained(path: &Path) -> bool {
        let mut depth: usize = 0;

        for component in path.components() {
            match component {
                Component::Normal(_) => depth += 1,
                Component::CurDir => (),
                Component::ParentDir if depth > 0 => depth -= 1,
                _ => return false,
            }
        }

        true
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        args.dot_config.ensure_dir()?;

        let dot_config = args.dot_config.canonicalize()?;
        let entries = fs::read_dir(&dot_config)?.collect::<io::Result<Vec<_>>>()?;

        let restore_file = match &self.from_file {
            Some(path) => path.clone(),
            None => entries
                .iter()
                .map(|entry| entry.path())
                .filter(|path| path.extension() == Some(OsStr::new("gz")) && path.is_file())
                .map(|path| -> Result<_> {
                    let modified = path.metadata()?.modified()?;

                    Ok((modified, path))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .max_by_key(|(modified, _)| *modified)
                .map(|(_, path)| path)
                .context("No backup file found")?,
        };

        restore_file.ensure_exists()?;

        let workspace = tempfile::tempdir_in(
            dot_config
                .parent()
                .context("Cannot restore filesystem root")?,
        )?;

        let staged = workspace.path().join("restored");

        fs::create_dir(&staged)?;

        let mut archive = Archive::new(GzDecoder::new(
            File::open(&restore_file)
                .with_context(|| format!("Failed to open {}", restore_file.display()))?,
        ));

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();

            if !Self::contained(&path) {
                bail!("Archive path escapes config directory: {}", path.display());
            }

            let kind = entry.header().entry_type();

            if !(kind.is_file() || kind.is_dir() || kind.is_symlink()) {
                bail!("Unsupported archive entry: {}", path.display());
            }

            if kind.is_symlink() {
                let target = entry.link_name()?.context("Symlink has no target")?;

                if !Self::contained(&path.parent().unwrap_or(Path::new("")).join(target)) {
                    bail!(
                        "Archive symlink escapes config directory: {}",
                        path.display()
                    );
                }
            }

            if !entry.unpack_in(&staged)? {
                bail!("Invalid archive path: {}", path.display());
            }
        }

        io::copy(&mut archive.into_inner(), &mut io::sink())
            .context("Backup gzip stream is corrupt or incomplete")?;

        for entry in entries {
            let path = entry.path();

            if path.extension() == Some(OsStr::new("gz")) {
                let target = staged.join(entry.file_name());

                if fs::symlink_metadata(&target).is_ok() {
                    bail!(
                        "Restored entry conflicts with preserved backup: {}",
                        target.display()
                    );
                }

                if !entry.file_type()?.is_file() {
                    bail!("Backup entry is not a regular file: {}", path.display());
                }

                fs::copy(&path, &target)?;
            }
        }

        let original = workspace.path().join("original");
        dot_config
            .move_dir(&original)
            .await
            .context("Failed to retain current configs")?;

        if let Err(error) = staged.move_dir(&dot_config).await {
            if let Err(rollback) = original.move_dir(&dot_config).await {
                let recovery = workspace.keep();

                bail!(
                    "Restore failed: {error}; rollback failed: {rollback}. Original configs retained at {}",
                    recovery.join("original").display()
                );
            }

            return Err(error).context("Failed to replace configs; original configs restored");
        }

        workspace
            .close()
            .context("Configs restored, but temporary directory cleanup failed")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::config::backup::Backup;
    use clap::Parser;

    fn setup() -> (tempfile::TempDir, CommandArgs) {
        let root = tempfile::tempdir().unwrap();
        let args = CommandArgs {
            dot_config: root.path().join(".config"),
            dotfiles_dir: root.path().join("home-config"),
        };

        fs::create_dir(&args.dot_config).unwrap();
        fs::create_dir(args.dot_config.join("editor")).unwrap();
        fs::write(args.dot_config.join("editor/settings"), "original").unwrap();
        fs::write(args.dot_config.join("plain"), "file contents").unwrap();

        (root, args)
    }

    #[tokio::test]
    async fn backup_restore_round_trip() {
        let (root, args) = setup();

        let archive = root.path().join("backup.tar.gz");

        Backup::parse_from(["backup", "--file", archive.to_str().unwrap()])
            .exec(&args)
            .await
            .unwrap();

        fs::write(args.dot_config.join("editor/settings"), "changed").unwrap();
        fs::write(args.dot_config.join("plain"), "changed").unwrap();

        Restore {
            from_file: Some(archive),
        }
        .exec(&args)
        .await
        .unwrap();

        assert_eq!(
            fs::read_to_string(args.dot_config.join("editor/settings")).unwrap(),
            "original"
        );

        assert_eq!(
            fs::read_to_string(args.dot_config.join("plain")).unwrap(),
            "file contents"
        );
    }

    #[tokio::test]
    async fn restore_preserves_symlinks() {
        let (_root, args) = setup();

        let link = args.dot_config.join("link");

        std::os::unix::fs::symlink("plain", &link).unwrap();

        Backup::parse_from(["backup"]).exec(&args).await.unwrap();

        fs::remove_file(&link).unwrap();

        Restore { from_file: None }.exec(&args).await.unwrap();

        link.ensure_symlink().unwrap();

        assert_eq!(fs::read_link(&link).unwrap(), Path::new("plain"));
        assert_eq!(fs::read_to_string(&link).unwrap(), "file contents");
    }
}
