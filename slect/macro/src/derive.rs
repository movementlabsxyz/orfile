use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote};
use syn::{
	parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, token::Comma,
	Data, DeriveInput, Expr, ExprLit, Fields, Lit, LitStr, Meta, Path,
};

/// Represents a single selection option, either with or without a prefix
#[derive(Debug)]
struct SelectionOption {
	prefix: Option<String>,
	subcommand: String,
}

/// Parse a selection option from the attribute
fn parse_selection_option(expr: &Expr) -> syn::Result<SelectionOption> {
	match expr {
		Expr::Tuple(tuple) => {
			if tuple.elems.len() != 2 {
				return Err(syn::Error::new_spanned(expr, "Expected (prefix, subcommand) tuple"));
			}

			let prefix = match &tuple.elems[0] {
				Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => Some(s.value()),
				_ => {
					return Err(syn::Error::new_spanned(
						&tuple.elems[0],
						"Expected string literal for prefix",
					))
				}
			};

			let subcommand = match &tuple.elems[1] {
				Expr::Path(p) => p
					.path
					.get_ident()
					.ok_or_else(|| syn::Error::new_spanned(p, "Expected identifier"))?
					.to_string(),
				_ => {
					return Err(syn::Error::new_spanned(
						&tuple.elems[1],
						"Expected identifier for subcommand",
					))
				}
			};

			Ok(SelectionOption { prefix, subcommand })
		}
		Expr::Path(p) => {
			let subcommand = p
				.path
				.get_ident()
				.ok_or_else(|| syn::Error::new_spanned(p, "Expected identifier"))?
				.to_string();
			Ok(SelectionOption { prefix: None, subcommand })
		}
		_ => Err(syn::Error::new_spanned(expr, "Expected path or tuple")),
	}
}

/// Generate the implementation for the struct
fn generate_impl(struct_name: &Ident, selections: Vec<SelectionOption>) -> TokenStream2 {
	let selection_variants = selections
		.iter()
		.map(|opt| {
			let variant_name = format_ident!("{}", opt.subcommand);
			quote! {
				#variant_name
			}
		})
		.collect::<Vec<_>>();

	let selection_names = selections.iter().map(|opt| &opt.subcommand).collect::<Vec<_>>();
	let selection_prefixes = selections
		.iter()
		.map(|opt| opt.prefix.as_deref().unwrap_or(""))
		.collect::<Vec<_>>();

	quote! {
		/// Represents the possible selections for this command
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		pub enum Selections {
			#(#selection_variants),*
		}

		impl #struct_name {
			/// Shows help for all possible subcommands
			pub fn help_all(&self) {
				use clap::CommandFactory;

				// Show help for the main command
				let mut cmd = <Self as CommandFactory>::command();
				cmd.print_help().unwrap();
				println!();

				// Show help for each subcommand
				#(
					println!("=== Help for {} ===", #selection_names);
					let mut cmd = <#selection_names as CommandFactory>::command();
					if !#selection_prefixes.is_empty() {
						cmd = cmd.name(format!("{}{}", #selection_prefixes, cmd.get_name()));
					}
					cmd.print_help().unwrap();
					println!();
				)*
			}

			/// Parse the extra_args into a selection
			pub fn select(&self) -> Option<Selections> {
				use clap::Parser;

				// Try parsing each subcommand
				#(
					if let Ok(_) = <#selection_names as Parser>::try_parse_from(self.extra_args.iter().map(|s| s.as_str())) {
						return Some(Selections::#selection_variants);
					}
				)*

				None
			}
		}
	}
}

pub fn impl_slect(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let struct_name = &input.ident;

	// Find the field with the #[select] attribute
	if let Data::Struct(data) = &input.data {
		if let Fields::Named(fields) = &data.fields {
			for field in &fields.named {
				if let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("select")) {
					let meta = attr
						.meta
						.require_list()
						.unwrap_or_else(|e| abort!(attr, "Expected a list of subcommands: {}", e));

					let exprs = meta
						.parse_args_with(Punctuated::<Expr, Comma>::parse_terminated)
						.unwrap_or_else(|e| abort!(meta, "Failed to parse subcommand list: {}", e));

					let selections = exprs
						.iter()
						.map(parse_selection_option)
						.collect::<syn::Result<Vec<_>>>()
						.unwrap_or_else(|e| abort!(exprs, "Failed to parse subcommand: {}", e));

					return generate_impl(struct_name, selections).into();
				}
			}
		}
	}

	abort_call_site!("Expected a struct with a field marked with #[select] attribute")
}
