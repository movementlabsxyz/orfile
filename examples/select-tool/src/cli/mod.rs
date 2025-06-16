pub mod add;
pub mod divide;
pub mod kebab_divide;
pub mod multiply;
use add::Add;
use divide::Divide;
use kebab_divide::KebabDivide;
use multiply::Multiply;
use select::Select;

use clap::Parser;

#[derive(Parser, Select)]
#[clap(rename_all = "kebab-case")]
pub struct Tool {
	#[select(add = Add, multiply = Multiply, divide = Divide, kebab_divide = KebabDivide)]
	extra_args: Vec<String>,
}

impl select_command::Tool {
	pub async fn execute(&self) -> Result<(), anyhow::Error> {
		let (maybe_add, maybe_multiply, maybe_divide, maybe_kebab_divide) =
			self.select().map_err(|e| anyhow::anyhow!(e))?;

		if let Some(add) = maybe_add {
			add.execute().await?;
		}

		if let Some(multiply) = maybe_multiply {
			multiply.execute().await?;
		}

		if let Some(divide) = maybe_divide {
			divide.execute().await?;
		}

		if let Some(kebab_divide) = maybe_kebab_divide {
			kebab_divide.execute().await?;
		}

		Ok(())
	}
}
