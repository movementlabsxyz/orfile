use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod derive;

#[proc_macro_derive(Select, attributes(select))]
#[proc_macro_error]
pub fn select_derive(input: TokenStream) -> TokenStream {
	derive::impl_select(input)
}
