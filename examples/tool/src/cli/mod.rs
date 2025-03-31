pub mod add;

use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Tool {
	#[clap(subcommand)]
	Add(add::or_file::Add),
}

impl Tool {
	pub async fn execute(&self) -> Result<(), anyhow::Error> {
		match self {
			Tool::Add(add) => {
				add.clone().resolve().await?.execute().await?;
			}
		}

		Ok(())
	}
}
