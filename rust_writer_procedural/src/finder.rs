// SPDX-License-Identifier: GPL-3.0

mod expand;

use crate::parse::{MacroAttrs, MacroImplParsed, MacroParsed};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

pub(crate) fn finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	let attrs_list = parse_macro_input!(attrs as MacroAttrs);
	let struct_def = parse_macro_input!(item as ItemStruct);

	match MacroParsed::try_from(attrs_list, struct_def) {
		Ok(parsed) => expand::expand_finder(parsed).into(),
		Err(err) => err.to_compile_error().into(),
	}
}

pub(crate) fn impl_finder(item: TokenStream) -> TokenStream {
	let mut finished = item.clone();

	let struct_def = parse_macro_input!(item as ItemStruct);

	let generated: TokenStream = match MacroImplParsed::try_from(struct_def, "found") {
		Ok(parsed) => expand::expand_impl_finder(parsed).into(),
		Err(err) => err.to_compile_error().into(),
	};

	finished.extend(generated);
	finished
}
