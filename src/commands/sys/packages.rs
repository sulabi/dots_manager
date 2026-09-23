use anyhow::Result;

// #[derive(clap::Parser)]
// pub struct Packages {
//     #[arg(long, default_value_t = false)]
//     list: bool,
//
//     #[arg(long, default_value_t = false)]
//     save: bool,
// }
#[derive(clap::Subcommand)]
pub enum Packages {
    List,
    Save,
}

use super::CommandArgs;

impl Packages {
    fn list() -> Result<Vec<String>> {
        let out = std::process::Command::new("sudo")
            .arg("pacman")
            .arg("-Qqe")
            .output()?;

        let packages: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .split("\n")
            .map(|x| x.into())
            .collect();

        Ok(packages)
    }

    pub async fn exec(&self, args: &CommandArgs) -> Result<()> {
        println!("getting all packages...");

        let packages = Packages::list()?;

        match self {
            Packages::List => {
                println!("{:?}", packages);
                println!("{} packages", packages.len());
            }
            Packages::Save => {
                println!("saving packages ..");
            }
        }

        Ok(())
    }
}
