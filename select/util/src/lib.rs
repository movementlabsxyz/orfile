use clap::builder::{IntoResettable, Resettable, StyledStr};
use once_cell::sync::Lazy;

/// A marker trait for select operations.
pub trait SelectOperations {
	/// Gets the help_selection_string text.
	///
	/// Note: this is useful for higher order compositions of Select commands.
	fn select_help_selection_string() -> String;
}

/// A lazy string that can be resettable.
///
/// Note: this is implemented as a wrapper type for Lazy s.t. we can implement
/// IntoResettable for it and use with #[command(after_help = LazyString::new(|| {
///     Self::help_selection_string()
/// }))]
pub struct LazyString<F: FnOnce() -> String>(Lazy<String, F>);

impl<F> LazyString<F>
where
	F: FnOnce() -> String,
{
	pub fn new(f: F) -> Self {
		Self(Lazy::new(f))
	}

	pub fn get(&self) -> String {
		Lazy::force(&self.0).clone()
	}
}

impl<F> IntoResettable<StyledStr> for LazyString<F>
where
	F: FnOnce() -> String,
{
	fn into_resettable(self) -> Resettable<StyledStr> {
		Resettable::from(StyledStr::from(self.get()))
	}
}
