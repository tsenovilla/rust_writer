// SPDX-License-Identiier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use proc_macro2::{Group, TokenStream, TokenTree};
use syn::{visit::Visit, visit_mut::VisitMut, Ident, Macro, Path};

#[derive(Debug, Clone)]
pub struct TokenStreamToMacro {
	pub macro_path: Path,
	pub container_ident: Option<Ident>,
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
