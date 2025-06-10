use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod derive;

#[proc_macro_derive(Slect, attributes(slect))]
#[proc_macro_error]
pub fn slect_derive(input: TokenStream) -> TokenStream {
	derive::impl_slect(input)
}
