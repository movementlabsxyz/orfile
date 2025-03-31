use clap::*;
use dotenv::dotenv;
use tool::cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
	// Load environment variables from .env file.
	dotenv().ok();

	// Run the CLI.
	let tool = cli::Tool::parse();
	tool.execute().await?;
	Ok(())
}
