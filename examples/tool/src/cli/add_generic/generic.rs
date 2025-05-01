use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use orfile::Orfile;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;

/// Common trait for add operations
pub trait AddOperation<T> {
	fn perform_add(left: T, right: T) -> T;
}

/// Implementation for numeric types
impl<T> AddOperation<T> for T
where
	T: std::ops::Add<Output = T> + Clone,
{
	fn perform_add(left: T, right: T) -> T {
		left + right
	}
}

/// The arguments for the add command
#[derive(Debug, Clone, Parser, Serialize, Deserialize)]
pub struct AddArgs {
	/// The left number
	#[clap(long)]
	pub left: String,
	/// The right number
	#[clap(long)]
	pub right: String,
}

/// The add command
#[derive(Debug, Clone, Parser, Serialize, Deserialize, Orfile)]
pub struct Add {
	#[orfile(config)]
	#[clap(flatten)]
	pub args: AddArgs,
}

impl Add {
	pub async fn execute<T>(&self) -> Result<(), anyhow::Error>
	where
		T: FromStr + Debug + Clone + std::ops::Add<Output = T> + AddOperation<T>,
		<T as FromStr>::Err: std::fmt::Display,
	{
		let left = self
			.args
			.left
			.parse::<T>()
			.map_err(|e| anyhow::anyhow!("Failed to parse left argument: {}", e))?;
		let right = self
			.args
			.right
			.parse::<T>()
			.map_err(|e| anyhow::anyhow!("Failed to parse right argument: {}", e))?;

		let result = T::perform_add(left.clone(), right.clone());
		println!("{:?} + {:?} = {:?}", left, right, result);

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_add() -> Result<(), anyhow::Error> {
		let add = Add { args: AddArgs { left: "1".to_string(), right: "2".to_string() } };

		add.execute::<i32>().await
	}

	#[tokio::test]
	async fn test_add_float() -> Result<(), anyhow::Error> {
		let add = Add { args: AddArgs { left: "1.5".to_string(), right: "2.5".to_string() } };

		add.execute::<f64>().await
	}
}
