use clap::Parser;
use orfile::Orfile;
use serde::{Deserialize, Serialize};

/// The arguments for the add command
///
/// We define this as a separate struct because Orfile requires separate config structs to allow composability and discretion between mandatory and `using` enabled fields.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[clap(rename_all = "kebab-case")]
pub struct AddArgs {
	/// The left number
	#[clap(long)]
	pub left: u64,
	/// The right number
	#[clap(long)]
	pub right: u64,
}

/// The add command
///
/// The [Orfile] macro derive will generate a module or_file with a struct [or_file::Add] which contains the `where` and `using` subcommands.

#[derive(Parser, Debug, Clone, Orfile)]
#[clap(rename_all = "kebab-case")]
pub struct Add {
	#[orfile(config)]
	#[clap(flatten)]
	pub args: AddArgs,
}

impl Add {
	pub async fn execute(&self) -> Result<(), anyhow::Error> {
		println!("{:?}", self);
		println!("{}", self.args.left + self.args.right);

		Ok(())
	}
}
