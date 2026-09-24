use anyhow::{Context, Result};

use tokio::{fs, process::Command};

use super::CommandArgs;

#[derive(clap::Subcommand)]
pub enum Fonts {
    List,
    Save,
}

impl Fonts {
    async fn list() -> Result<Vec<String>> {
        let out = Command::new("fc-list")
            .args(["--format", "%{family}\n"])
            .output()
            .await
            .context("Failed to get fonts")?;

        if !out.status.success() {
            anyhow::bail!(
                "fc-list excited with {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            )
        }

        let mut fonts: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect();

        fonts.sort();
        fonts.dedup();

        Ok(fonts)
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        println!("Getting all fonts...");

        let fonts = Fonts::list().await?;

        match self {
            Self::List => {
                for font in &fonts {
                    println!("{font}");
                }

                println!("there are {} fonts", fonts.len());
            }

            Self::Save => {
                let file = args.dots_directory.join("fonts.txt");

                fs::write(&file, fonts.join("\n"))
                    .await
                    .with_context(|| format!("Failed to save fonts to {}", file.display()))?;

                println!("Saved all fonts to {}!", file.display());
            }
        }

        Ok(())
    }
}
