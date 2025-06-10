pub mod add;
pub mod divide;
pub mod multiply;
use add::Add;
use divide::Divide;
use multiply::Multiply;
use slect::Slect;

use clap::Parser;

#[derive(Parser, Slect)]
#[clap(rename_all = "kebab-case")]
pub struct Tool {
	#[slect(add = Add, multiply = Multiply, divide = Divide)]
	extra_args: Vec<String>,
}

impl select::Tool {
	pub async fn execute(&self) -> Result<(), anyhow::Error> {
		let (maybe_add, maybe_multiply, maybe_divide) = self.select();

		if let Some(add) = maybe_add {
			add.execute().await?;
		}

		if let Some(multiply) = maybe_multiply {
			multiply.execute().await?;
		}

		if let Some(divide) = maybe_divide {
			divide.execute().await?;
		}

		Ok(())
	}
}
