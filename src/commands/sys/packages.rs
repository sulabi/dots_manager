use anyhow::{Context, Result};
use log::info;
use tokio::{fs, process::Command};

#[derive(clap::Subcommand)]
pub enum Packages {
    List,
    Save,
}

use crate::commands::PathValidate;

use super::CommandArgs;

// TODO: use toml format for packages file and add these
enum _PackageType {
    Pacman,
    Pip,
    Cargo,
}

impl Packages {
    async fn list() -> Result<Vec<String>> {
        let out = Command::new("pacman").arg("-Qqe").output().await?;

        if !out.status.success() {
            anyhow::bail!(
                "pacman excited with {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            )
        }

        let packages: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();

        Ok(packages)
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        info!("getting all packages...");

        let mut packages = Packages::list().await?;
        packages.sort();

        match self {
            Packages::List => {
                for pkg in &packages {
                    println!("{pkg}")
                }
                println!("{}, packages", packages.len());
            }
            Packages::Save => {
                args.dots_directory.ensure_exists()?;

                let file = args.dots_directory.join("packages.txt");
                fs::write(&file, packages.join("\n"))
                    .await
                    .context("Failed to write packages to packages.txt")?;

                info!("Saved packages to {}!", file.display());
            }
        }

        Ok(())
    }
}
