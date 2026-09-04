mod add;
mod backup;
mod install;
mod remove;

use anyhow::{Context, Result, anyhow, bail};
use std::path::{Path, PathBuf};
use std::pin::Pin;

pub use add::*;
pub use backup::*;
pub use install::*;
pub use remove::*;

pub trait PathValidate {
    fn ensure_dir(&self) -> Result<&Self>;
    fn ensure_child(&self, parent_dir: &Path) -> Result<&Self>;
    fn ensure_symlink(&self) -> Result<&Self>;
}

impl PathValidate for Path {
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

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait PathExt {
    fn relative_path(&self, base: &Path) -> Result<PathBuf>;
    fn move_dir<'a>(&'a self, target: &'a Path) -> BoxFuture<'a, Result<()>>;
}

impl PathExt for Path {
    fn relative_path(&self, base: &Path) -> Result<PathBuf> {
        let resolved = self
            .canonicalize()
            .with_context(|| format!("Faled to resolve {:?}", self))?;

        pathdiff::diff_paths(resolved, base).ok_or_else(|| anyhow!("Failed to get relative path"))
    }

    fn move_dir<'a>(&'a self, target: &'a Path) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            tokio::fs::rename(self, target)
                .await
                .with_context(|| format!("Unable to move folder {:?} into {:?}", self, target))?;

            Ok(())
        })
    }
}
