use clap::*;
use dotenv::dotenv;
use select_tool::cli::select_command;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
	// Load environment variables from .env file.
	dotenv().ok();

	// Run the CLI.
	let tool = select_command::Tool::parse();
	tool.execute().await?;
	Ok(())
}
