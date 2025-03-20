// SPDX-License-Identifier: GPL-3.0

mod finder;
pub(crate) mod helpers;
mod mutator;
pub(crate) mod parse;

use proc_macro::TokenStream;

/// # Description
///
/// The mutator macro is used to define a new implementor which combines a  
///
/// # Compatibility with [`#[finder]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.finder.html)
#[proc_macro_attribute]
pub fn mutator(attrs: TokenStream, item: TokenStream) -> TokenStream {
	mutator::mutator(attrs, item)
}

#[proc_macro_attribute]
pub fn local_mutator(_: TokenStream, item: TokenStream) -> TokenStream {
	mutator::local_mutator(item)
}

#[proc_macro_attribute]
pub fn finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	finder::finder(attrs, item)
}

#[proc_macro_attribute]
pub fn local_finder(attrs: TokenStream, item: TokenStream) -> TokenStream {
	finder::local_finder(attrs, item)
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn already_expanded(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}
