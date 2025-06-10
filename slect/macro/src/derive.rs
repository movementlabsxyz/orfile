use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{
	parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, token::Comma,
	Data, DeriveInput, Expr, ExprLit, ExprPath, FieldMutability, Fields, Lit, Meta, MetaNameValue,
	Path, Result as SynResult, Token,
};

/// A selection option parsed from the attribute
struct SelectionOption {
	/// The name of the flag (e.g., "add" for --add)
	flag_name: Ident,
	/// The type of the subcommand (e.g., Add)
	subcommand_type: Box<Path>,
}

/// Get the last segment of a path as an identifier
fn get_path_ident(path: &Path) -> syn::Result<Ident> {
	Ok(path
		.segments
		.last()
		.ok_or_else(|| syn::Error::new_spanned(path, "Expected a non-empty path"))?
		.ident
		.clone())
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

			let subcommand_path = match &tuple.elems[1] {
				Expr::Path(p) => p.path.clone(),
				_ => {
					return Err(syn::Error::new_spanned(
						&tuple.elems[1],
						"Expected path for subcommand",
					))
				}
			};

			Ok(SelectionOption {
				flag_name: get_path_ident(&subcommand_path)?,
				subcommand_type: Box::new(subcommand_path),
			})
		}
		Expr::Path(p) => Ok(SelectionOption {
			flag_name: get_path_ident(&p.path)?,
			subcommand_type: Box::new(p.path.clone()),
		}),
		_ => Err(syn::Error::new_spanned(expr, "Expected path or tuple")),
	}
}

/// Generate the struct definition with the extra_args field
fn generate_struct(struct_name: &Ident, selections: &[SelectionOption]) -> TokenStream2 {
	let selection_paths = selections.iter().map(|opt| &opt.subcommand_type).collect::<Vec<_>>();

	quote! {
		#[derive(Debug, Parser)]
		#[command(name = "select-tool")]
		/// A tool for selecting and running subcommands with optional prefixes.
		///
		/// Extra arguments can be passed to the subcommands. These can be prefixed with
		/// the subcommand's prefix (e.g., --add.arg1, --multiply.arg2) or unprefixed.
		///
		/// Available flags:
		/// --help-all: Show help for all possible subcommands
		/// --add: Enable the add subcommand
		/// --multiply: Enable the multiply subcommand
		/// --divide: Enable the divide subcommand
		pub struct #struct_name {
			/// Extra arguments to be passed to subcommands
			#[arg(last = true)]
			pub extra_args: Vec<String>,

			/// Show help for all possible subcommands
			#[arg(long)]
			pub help_all: bool,

			#(
				/// Enable the #selection_paths subcommand
				#[arg(long)]
				pub #selection_paths: bool,
			)*
		}
	}
}

/// Generate the implementation for the struct
fn generate_impl(struct_name: &Ident, selections: &[SelectionOption]) -> TokenStream2 {
	let selection_paths = selections.iter().map(|opt| &opt.subcommand_type).collect::<Vec<_>>();
	let flag_names = selections.iter().map(|opt| &opt.flag_name).collect::<Vec<_>>();

	// Generate the prefix handling code for each selection
	let prefix_handlers: Vec<_> = selections
		.iter()
		.map(|opt| {
			let ty = &opt.subcommand_type;
			let flag = &opt.flag_name;
			quote! {
				{
					if self.#flag {
						const PREFIX: &str = concat!("--", stringify!(#flag), ".");
						let subcommand_args: Vec<_> = self.extra_args.iter()
							.filter(|arg| arg.starts_with(PREFIX))
							.cloned()
							.collect();

						if !subcommand_args.is_empty() {
							<#ty as Parser>::try_parse_from(subcommand_args.iter().map(|s| s.as_str())).ok()
						} else {
							None
						}
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
		.map(|opt| {
			let ty = &opt.subcommand_type;
			let flag = &opt.flag_name;
			quote! {
				{
					println!("=== Help for {} ===", <#ty as CommandFactory>::command().get_name());
					let mut cmd = <#ty as CommandFactory>::command();
					cmd = cmd.name(concat!(stringify!(#flag), "{}"));
					cmd.print_help().unwrap();
					println!();
				}
			}
		})
		.collect();

	let return_type = if selections.len() == 1 {
		let ty = &selection_paths[0];
		quote! { Option<#ty> }
	} else {
		let types: Vec<_> = selection_paths.iter().map(|ty| quote! { Option<#ty> }).collect();
		quote! { (#(#types),*) }
	};

	let return_value = if selections.len() == 1 {
		let ty = &selection_paths[0];
		quote! { #ty }
	} else {
		let values: Vec<_> = selection_paths.iter().map(|ty| quote! { #ty }).collect();
		quote! { (#(#values),*) }
	};

	quote! {
		impl #struct_name {
			/// Shows help for all possible subcommands
			pub fn help_all(&self) {
				if self.help_all {
					use clap::CommandFactory;

					// Show help for the main command
					let mut cmd = <Self as CommandFactory>::command();
					cmd.print_help().unwrap();
					println!();

					// Show help for each subcommand
					#(#help_handlers)*
				}
			}

			/// Parse the extra_args into selections for each subcommand
			pub fn select(&self) -> #return_type {
				use clap::Parser;

				// Show help if requested
				self.help_all();

				// Try parsing each subcommand
				#(
					let #selection_paths = #prefix_handlers;
				)*

				#return_value
			}
		}
	}
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
					let module_def = generate_module(&module_name, &struct_name, &selections);

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
) -> TokenStream2 {
	let selection_types = selections.iter().map(|opt| &*opt.subcommand_type).collect::<Vec<_>>();
	let flag_names = selections.iter().map(|opt| &opt.flag_name).collect::<Vec<_>>();

	// Generate the prefix handling code for each selection
	let prefix_handlers: Vec<_> = selections
		.iter()
		.map(|opt| {
			let ty = &opt.subcommand_type;
			let flag = &opt.flag_name;
			quote! {
				{
					if self.#flag {
						const PREFIX: &str = concat!("--", stringify!(#flag), ".");
						let subcommand_args: Vec<_> = self.extra_args.iter()
							.filter(|arg| arg.starts_with(PREFIX))
							.cloned()
							.collect();

						if !subcommand_args.is_empty() {
							<#ty as Parser>::try_parse_from(subcommand_args.iter().map(|s| s.as_str())).ok()
						} else {
							None
						}
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
		.map(|opt| {
			let ty = &opt.subcommand_type;
			let flag = &opt.flag_name;
			quote! {
				{
					println!("=== Help for {} ===", <#ty as CommandFactory>::command().get_name());
					let mut cmd = <#ty as CommandFactory>::command();
					cmd = cmd.name(concat!(stringify!(#flag), "{}"));
					cmd.print_help().unwrap();
					println!();
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

	let return_value = if selections.len() == 1 {
		let ty = &selection_types[0];
		quote! { #ty }
	} else {
		let values: Vec<_> = selection_types.iter().map(|ty| quote! { #ty }).collect();
		quote! { (#(#values),*) }
	};

	quote! {
		pub mod #module_name {
			use super::*;
			use clap::{Parser, CommandFactory};

			/// A wrapper struct that adds selection flags to the original struct
			#[derive(Debug, Parser)]
			#[command(name = "select-tool")]
			pub struct #struct_name {
				/// Extra arguments to be passed to subcommands
				#[arg(last = true)]
				pub extra_args: Vec<String>,

				/// Show help for all possible subcommands
				#[arg(long)]
				pub help_all: bool,

				#(
					/// Enable the #flag_names subcommand
					#[arg(long)]
					pub #flag_names: bool,
				)*
			}

			impl #struct_name {
				/// Shows help for all possible subcommands
				pub fn help_all(&self) {
					if self.help_all {
						// Show help for the main command
						let mut cmd = <super::#struct_name as CommandFactory>::command();
						cmd.print_help().unwrap();
						println!();

						// Show help for each subcommand
						#(#help_handlers)*
					}
				}

				/// Parse the extra_args into selections for each subcommand
				pub fn select(&self) -> #return_type {
					// Show help if requested
					self.help_all();

					// Try parsing each subcommand
					#(
						let #selection_types = #prefix_handlers;
					)*

					#return_value
				}
			}

			impl super::#struct_name {
				/// Create a new selector for this struct
				pub fn selector(&self) -> #struct_name {
					#struct_name {
						extra_args: self.extra_args.clone(),
						help_all: false,
						#(
							#flag_names: false,
						)*
					}
				}
			}
		}
	}
}
