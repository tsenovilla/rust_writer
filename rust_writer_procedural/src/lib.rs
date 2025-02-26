// SPDX-License-Identifier: GPL-3.0

mod finder;
pub(crate) mod helpers;
mod mutator;
pub(crate) mod parse;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn mutator(attrs: TokenStream, item: TokenStream) -> TokenStream {
	mutator::mutator(attrs, item)
}

#[proc_macro_attribute]
pub fn impl_mutator(_: TokenStream, item: TokenStream) -> TokenStream {
	mutator::impl_mutator(item)
}

#[proc_macro_attribute]
pub fn finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	finder::finder(attrs, item)
}

#[proc_macro_attribute]
pub fn impl_finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	finder::impl_finder(attrs, item)
}

#[proc_macro_attribute]
pub fn already_expanded(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}

#[proc_macro_attribute]
pub fn already_impl_from(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}
