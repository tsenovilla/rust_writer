// SPDX-License-Identiier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use proc_macro2::{Group, TokenStream, TokenTree};
use syn::{visit::Visit, visit_mut::VisitMut, Ident, Macro, Path};

/// This implementor targets any [`TokenStream`](https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html)
/// inside a declarative macro.
///
/// When used inside a [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html)
/// It works in a "syntactical" way, meaning that it consider the `TokenStream` found if the same
/// tokens identifiers are contained in the macro in the same order. This means that other parsing
/// details such as spans or spacing are ignored by this implementor. Have a look at
/// [this function](https://docs.rs/rustilities/latest/rustilities/parsing/fn.syntactic_token_stream_contains.html)
/// for further details, as it's used internally.
#[derive(Debug, Clone)]
pub struct TokenStreamToMacro {
	/// The path used to invoke the macro in the AST. Eg, the `println` in `println!("hello")`.
	pub macro_path: Path,
	/// If specified, the implementor will look inside a block preceded by this ident.
	/// Imagine this macro invocation:
	///
	/// ```no_compile
	/// macro_call!{
	///   let a = 1;
	///   some super useful group{
	///     Token1, Token2 ; Token 3
	///   }
	/// }
	/// ```
	///
	/// If container_ident is `None`, the implementor will target the whole
	/// macro invocation. If container_ident is `Some(group)`, it'll only target the inner group.
	///
	/// The inner `Ident` **must** be the token just before the group start, so if container_ident
	/// is `Some(some)`, the implementor will target the whole macro invocation.
	///
	/// This currently supports only one level of depth; that is, can only target groups directly
	/// defined under the macro invocation, but not those under another group.
	pub container_ident: Option<Ident>,
	/// The target `TokenStream`.
	pub token_stream: TokenStream,
}

impl From<(Path, Option<Ident>, TokenStream)> for TokenStreamToMacro {
	fn from(tuple: (Path, Option<Ident>, TokenStream)) -> Self {
		Self { macro_path: tuple.0, container_ident: tuple.1, token_stream: tuple.2 }
	}
}

impl<'a> ToFind<'a, TokenStreamToMacro, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a TokenStreamToMacro) -> Finder<'a, TokenStreamToMacro, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, TokenStreamToMacro, 1> {
	fn visit_macro(&mut self, macro_: &'a Macro) {
		if self.finder.token_stream.is_empty() {
			return;
		}

		if macro_.path == self.finder.macro_path {
			match self.finder.container_ident.clone() {
				Some(ident) => {
					let macro_tokens_iter = macro_.tokens.clone().into_iter();
					let mut ident_found = false;
					for token in macro_tokens_iter {
						match token {
							TokenTree::Ident(macro_ident)
								if ident == macro_ident && !ident_found =>
								ident_found = true,
							TokenTree::Group(group)
								if ident_found &&
									rustilities::parsing::syntactic_token_stream_contains(
										self.finder.token_stream.clone(),
										group.stream().clone(),
									) =>
							{
								self.found[0] = true;
								break;
							},
							_ if ident_found => ident_found = false,
							_ => continue,
						}
					}
				},
				None if rustilities::parsing::syntactic_token_stream_contains(
					self.finder.token_stream.clone(),
					macro_.tokens.clone(),
				) =>
					self.found[0] = true,
				_ => (),
			}
		}
	}
}

impl<'a> ToMutate<'a, TokenStreamToMacro, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a TokenStreamToMacro) -> Mutator<'a, TokenStreamToMacro, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl VisitMut for Mutator<'_, TokenStreamToMacro, 1> {
	fn visit_macro_mut(&mut self, macro_: &mut Macro) {
		if macro_.path == self.mutator.macro_path {
			match self.mutator.container_ident.clone() {
				Some(ident) => {
					let mut new_tokens = TokenStream::new();
					let mut ident_found = false;
					for token in macro_.tokens.clone().into_iter() {
						match token {
							TokenTree::Ident(macro_ident)
								if ident == macro_ident && !ident_found =>
							{
								ident_found = true;
								new_tokens.extend(Some(TokenTree::Ident(macro_ident)));
							},
							TokenTree::Group(group) if ident_found => {
								let mut group_stream = group.stream();
								group_stream.extend(self.mutator.token_stream.clone());
								new_tokens.extend(Some(TokenTree::Group(Group::new(
									group.delimiter(),
									group_stream,
								))));
								self.mutated[0] = true;
								ident_found = false;
							},
							_ if ident_found => {
								ident_found = false;
								new_tokens.extend(Some(token));
							},
							token => {
								new_tokens.extend(Some(token));
							},
						}
					}
					macro_.tokens = new_tokens;
				},
				None => {
					macro_.tokens.extend(self.mutator.token_stream.clone());
					self.mutated[0] = true;
				},
			}
		}
	}
}
