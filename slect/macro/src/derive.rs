use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{ parse_macro_input, punctuated::Punctuated, token::Comma,
	Data, DeriveInput, Expr, ExprPath, Fields, MetaNameValue,
	Path,
};

/// A selection option parsed from the attribute
struct SelectionOption {
	/// The name of the flag (e.g., "add" for --add)
	flag_name: Ident,
	/// The type of the subcommand (e.g., Add)
	subcommand_type: Box<Path>,
}

/// Implementation of the derive macro
pub fn impl_slect(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let struct_name = input.ident.clone();
	let module_name = quote::format_ident!("select");

	// Find the field with the #[slect] attribute
	if let Data::Struct(data) = &input.data {
		if let Fields::Named(fields) = &data.fields {
			for field in &fields.named {
				if let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("slect")) {
					let meta = attr
						.meta
						.require_list()
						.unwrap_or_else(|e| abort!(attr, "Expected a list of subcommands: {}", e));

					let selections = meta
						.parse_args_with(Punctuated::<MetaNameValue, Comma>::parse_terminated)
						.unwrap_or_else(|e| abort!(meta, "Failed to parse subcommand list: {}", e))
						.into_iter()
						.map(|meta| {
							let flag_name = meta.path.get_ident().unwrap_or_else(|| {
								abort!(meta, "Expected an identifier for flag name")
							});

							let subcommand_type = match &meta.value {
								Expr::Path(ExprPath { path, .. }) => Box::new(path.clone()),
								_ => abort!(meta, "Expected a type path for subcommand"),
							};

							SelectionOption { flag_name: flag_name.clone(), subcommand_type }
						})
						.collect::<Vec<_>>();

					// Generate the module with the struct and implementation
					let module_def = generate_module(&module_name, &struct_name, &selections, &input);

					// Return the module definition
					return TokenStream::from(module_def);
				}
			}
		}
	}

	abort_call_site!("Expected a struct with a field marked with #[slect] attribute")
}

/// Generate a module containing the struct and implementation
fn generate_module(
	module_name: &Ident,
	struct_name: &Ident,
	selections: &[SelectionOption],
	_input: &DeriveInput,
) -> TokenStream2 {
	let selection_types = selections.iter().map(|opt| &*opt.subcommand_type).collect::<Vec<_>>();
	let flag_names = selections.iter().map(|opt| &opt.flag_name).collect::<Vec<_>>();
	let selections_len = selections.len();

	// Generate the prefix handling code for each selection
	let prefix_handlers: Vec<_> = selections
		.iter()
		.map(|opt| {
			let ty = &opt.subcommand_type;
			let flag = &opt.flag_name;
			quote! {
				{
					if self.#flag {
						const LONG_PREFIX: &str = concat!("--", stringify!(#flag), ".");
						const SHORT_PREFIX: &str = concat!("-", stringify!(#flag), ".");
						let mut subcommand_args = Vec::new();
						let mut args_iter = self.extra_args.iter().peekable();
						
						while let Some(arg) = args_iter.next() {
							if arg.starts_with(LONG_PREFIX) {
								// Handle prefixed arguments (--flag.value)
								subcommand_args.push(format!("--{}", arg.replace(LONG_PREFIX, "")));

								// if the next argument does not strart with -, it is a value for this flag
								if let Some(next_arg) = args_iter.peek() {
									if !next_arg.starts_with('-') {
										subcommand_args.push(args_iter.next().unwrap().clone());
									}
								}

							} else if arg.starts_with(SHORT_PREFIX) {
								// Handle prefixed arguments (-flag.value)
								subcommand_args.push(format!("-{}", arg.replace(SHORT_PREFIX, "")));

								// if the next argument does not strart with -, it is a value for this flag
								if let Some(next_arg) = args_iter.peek() {
									if !next_arg.starts_with('-') {
										subcommand_args.push(args_iter.next().unwrap().clone());
									}
								}
							}
						}
						
						// Add the program name as the first argument (required by clap)
						// this "subcommand" name is just temporary; it doesn't matter what it is
						let mut args = vec![stringify!(#flag).to_string()];
						args.extend(subcommand_args);
						
						Some(<#ty as Parser>::try_parse_from(args.iter().map(|s| s.as_str())).map_err(|e| format!("Failed to parse subcommand: {}", e))?)
						
					} else {
						None
					}
				}
			}
		})
		.collect();

	// Generate the help display code for each selection
	let help_handlers: Vec<_> = selections
		.iter()
		.enumerate()
		.map(|(index, opt)| {
			let ty = &opt.subcommand_type;
			let flag = &opt.flag_name;
			quote! {
				{
					let mut help = String::new();
					help.push_str(&format!("\x1b[1;4mSelection ({}/{}):\x1b[0m {}\n", #index + 1, #selections_len, stringify!(#flag)));
					let mut cmd = <#ty as CommandFactory>::command();
					cmd = cmd.name(concat!(stringify!(#flag), "{}"));
					let mut help_buf = Vec::new();
					cmd.write_help(&mut help_buf).unwrap();
					help.push_str(&String::from_utf8_lossy(&help_buf));
					help.push_str("\n");
					help
				}
			}
		})
		.collect();

	let return_type = if selections.len() == 1 {
		let ty = &selection_types[0];
		quote! { Option<#ty> }
	} else {
		let types: Vec<_> = selection_types.iter().map(|ty| quote! { Option<#ty> }).collect();
		quote! { (#(#types),*) }
	};

	quote! {
		pub mod #module_name {
			use super::*;
			use clap::{Parser, CommandFactory};
			use slect::SlectOperations;
			use slect::LazyString;

			/// A wrapper struct that adds selection flags to the original struct
			#[derive(Debug, Parser)]
			#[command(after_help = LazyString::new(|| {
				#struct_name::help_selection_string()
			}))]
			pub struct #struct_name {
				/// Extra arguments to be passed to subcommands
				#[arg(last = true)]
				pub extra_args: Vec<String>,

				#(
					/// Enable the #flag_names subcommand
					#[arg(long)]
					pub #flag_names: bool,
				)*
			}

			impl #struct_name {
				/// Get help text for all possible subcommands
				pub fn help_selection_string() -> String {
					let mut help = String::new();

					// Show help for each subcommand
					#(
						help.push_str(&#help_handlers);
					)*

					help
				}

				/// Parse the extra_args into selections for each subcommand
				pub fn select(&self) -> Result<#return_type, String> {

					// Try parsing each subcommand
					#(
						let #flag_names: Option<#selection_types> = #prefix_handlers;
					)*

					Ok((#(#flag_names),*))
				}
			}

			impl SlectOperations for #struct_name {
				fn select_help_selection_string() -> String {
					Self::help_selection_string()
				}
			}
		}
	}
}
