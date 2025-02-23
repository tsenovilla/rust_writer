// SPDX-License-Identifier: GPL-3.0

mod expand;
mod parse;

use crate::parse_attrs::MacroAttrs;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

pub(crate) fn mutator(attrs: TokenStream, item: TokenStream) -> TokenStream {
	let attrs_list = parse_macro_input!(attrs as MacroAttrs);
	let struct_def = parse_macro_input!(item as ItemStruct);

	match parse::MutatorDef::try_from(attrs_list, struct_def) {
		Ok(def) => expand::expand_mutator(def).into(),
		Err(err) => err.to_compile_error().into(),
	}
}
