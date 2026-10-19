use anyhow::{Context, Result, anyhow, bail};
use std::path::{Component, Path, PathBuf};

pub trait PathValidate {
    fn ensure_exists(&self) -> Result<&Self>;
    fn ensure_dir(&self) -> Result<&Self>;
    fn ensure_child(&self, parent_dir: &Path) -> Result<&Self>;
    fn ensure_symlink(&self) -> Result<&Self>;
}

impl PathValidate for Path {
    fn ensure_exists(&self) -> Result<&Self> {
        if !self.is_file() && !self.is_dir() {
            bail!("{:?}", self);
        }
        Ok(self)
    }

    fn ensure_dir(&self) -> Result<&Self> {
        if !self.is_dir() {
            bail!("{:?} is not a valid directory", self);
        }
        Ok(self)
    }

    fn ensure_symlink(&self) -> Result<&Self> {
        if !self.is_symlink() {
            bail!("{:?} is not a symlink", self);
        }
        Ok(self)
    }

    fn ensure_child(&self, parent_dir: &Path) -> Result<&Self> {
        let canonical_self = self
            .canonicalize()
            .with_context(|| format!("Failed to resolve {:?}", self))?;

        let canonical_parent = parent_dir
            .canonicalize()
            .with_context(|| format!("Failed to resolve parent directory {:?}", parent_dir))?;

        if canonical_self.parent() != Some(canonical_parent.as_path()) {
            bail!("{:?} is not in the {:?} folder ", self, parent_dir);
        }

        Ok(self)
    }
}

pub trait PathExt {
    /// Resolve a name or path to a direct entry in the given directory.
    fn selected_path(&self, base: &Path) -> Result<PathBuf>;
    /// Check a symlink target, including links to missing entries.
    fn link_matches(&self, file: &Path) -> Result<bool>;
    /// Returns the relative path from base
    fn relative_path(&self, base: &Path) -> Result<PathBuf>;
    /// Moves the whole directory to the target path
    async fn move_dir<'a>(&'a self, target: &'a Path) -> Result<()>;
}

impl PathExt for Path {
    fn selected_path(&self, base: &Path) -> Result<PathBuf> {
        let base = base.ensure_dir()?.canonicalize()?;

        let path = if self.components().count() == 1
            && matches!(self.components().next(), Some(Component::Normal(_)))
        {
            base.join(self)
        } else {
            self.to_path_buf()
        };

        let parent = path.parent().context("Missing config parent directory")?;

        if parent.ensure_dir()?.canonicalize()? != base {
            bail!(
                "{} is not directly inside {}",
                self.display(),
                base.display()
            );
        }

        let name = path.file_name().context("Missing config file name")?;

        Ok(base.join(name))
    }

    fn link_matches(&self, file: &Path) -> Result<bool> {
        self.ensure_symlink()?;

        let parent = self.parent().context("Missing symlink parent directory")?;
        let target = parent.join(std::fs::read_link(self)?);

        match (target.canonicalize(), file.canonicalize()) {
            (Ok(target), Ok(file)) => Ok(target == file),
            _ => {
                let target_parent = target.parent().context("Missing symlink target parent")?;
                let file_parent = file.parent().context("Missing managed config parent")?;

                Ok(target.file_name() == file.file_name()
                    && target_parent
                        .canonicalize()
                        .ok()
                        .zip(file_parent.canonicalize().ok())
                        .is_some_and(|(target, file)| target == file))
            }
        }
    }

    fn relative_path(&self, base: &Path) -> Result<PathBuf> {
        let resolved = self
            .canonicalize()
            .with_context(|| format!("Faled to resolve {:?}", self))?;

        pathdiff::diff_paths(resolved, base).ok_or_else(|| anyhow!("Failed to get relative path"))
    }

    async fn move_dir<'a>(&'a self, target: &'a Path) -> Result<()> {
        tokio::fs::rename(self, target)
            .await
            .with_context(|| format!("Unable to move folder {:?} into {:?}", self, target))?;

        Ok(())
    }
}
