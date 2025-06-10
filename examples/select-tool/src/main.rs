use clap::*;
use dotenv::dotenv;
use select_tool::cli::select;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
	// Load environment variables from .env file.
	dotenv().ok();

	// Run the CLI.
	let tool = select::Tool::parse();
	tool.execute().await?;
	Ok(())
}
