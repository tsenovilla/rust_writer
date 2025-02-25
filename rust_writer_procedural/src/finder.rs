// SPDX-License-Identifier: GPL-3.0

mod expand;
mod parse;

use crate::parse_attrs::MacroAttrs;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

pub(crate) fn finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	let attrs_list = parse_macro_input!(attrs as MacroAttrs);
	let struct_def = parse_macro_input!(item as ItemStruct);

	match parse::FinderDef::try_from(attrs_list, struct_def) {
		Ok(def) => expand::expand_finder(def).into(),
		Err(err) => err.to_compile_error().into(),
	}
}

pub(crate) fn impl_finder(item: TokenStream) -> TokenStream {
	let mut finished = item.clone();

	let struct_def = parse_macro_input!(item as ItemStruct);

	let generated: TokenStream = match parse::ImplFinderDef::try_from(struct_def) {
		Ok(def) => expand::expand_impl_finder(def).into(),
		Err(err) => err.to_compile_error().into(),
	};

	finished.extend(generated);
	finished
}
