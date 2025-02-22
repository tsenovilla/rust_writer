// SPDX-License-Identifier: GPL-3.0

mod mutator;
pub(crate) mod parse_attrs;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn mutator(attrs: TokenStream, item: TokenStream) -> TokenStream {
	mutator::mutator(attrs, item)
}

#[proc_macro_attribute]
pub fn already_expanded(_: TokenStream, input: TokenStream) -> TokenStream {
	input
}
