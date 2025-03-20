// SPDX-License-Identifier: GPL-3.0

mod expand;

use crate::parse::{MacroAttrs, MacroFinderMutatorParsed, MacroImplParsed};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

pub(crate) fn mutator(attrs: TokenStream, item: TokenStream) -> TokenStream {
	let attrs_list = parse_macro_input!(attrs as MacroAttrs);
	let struct_def = parse_macro_input!(item as ItemStruct);

	match MacroFinderMutatorParsed::try_from(attrs_list, struct_def) {
		Ok(parsed) => expand::expand_mutator(parsed).into(),
		Err(err) => err.to_compile_error().into(),
	}
}

pub(crate) fn local_mutator(item: TokenStream) -> TokenStream {
	let mut finished = item.clone();

	let struct_def = parse_macro_input!(item as ItemStruct);

	let generated: TokenStream = match MacroImplParsed::try_from(struct_def, "mutated") {
		Ok(parsed) => expand::expand_local_mutator(parsed).into(),
		Err(err) => err.to_compile_error().into(),
	};

	finished.extend(generated);
	finished
}
