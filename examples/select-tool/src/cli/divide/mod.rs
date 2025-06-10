use clap::Parser;
use serde::{Deserialize, Serialize};

/// The arguments for the divide command
///
/// We define this as a separate struct because Orfile requires separate config structs to allow composability and discretion between mandatory and `using` enabled fields.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
pub struct Divide {
	/// The left number
	#[clap(long)]
	pub left: u64,
	/// The right number
	#[clap(long)]
	pub right: u64,
}

impl Divide {
	pub async fn execute(&self) -> Result<(), anyhow::Error> {
		println!("{:?}", self);
		println!("{}", self.left / self.right);

		Ok(())
	}
}
