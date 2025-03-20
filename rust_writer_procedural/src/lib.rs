// SPDX-License-Identifier: GPL-3.0

mod finder;
pub(crate) mod helpers;
mod mutator;
pub(crate) mod parse;

use proc_macro::TokenStream;

/// # Description
///
/// The `#[mutator]` macro is used to define a new implementor which combines other implementors,
/// capable of mutating different parts of an AST with just one instruction, with the same level of
/// precision as if we had used each implementor separately.
///
/// The syntax to use the `#[mutator]` macro is pretty simple:
/// 
/// ```rust
/// use rust_writer::ast::{mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::visit_mut::VisitMut;
///
/// // The new implementor behaves as two `ItemToTrait` implementors.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>)]
/// struct NewImplementor;
/// ```
///
/// From the snippet above we can already assert that the `#[mutator]` macro needs that the traits 
/// [`ToMutate`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/trait.ToMutate.html)
/// and [`VisitMut`](https://docs.rs/syn/latest/syn/visit_mut/trait.VisitMut.html) need to be
/// brought into scope.
///
/// It's also possible to combine an implementor from the predefined set with a local implementor,
/// just by adding the keyword `local` before the local implementor.
///
/// ```rust
/// use rust_writer::ast::{mutator, local_mutator, implementors::ItemToTrait, mutator::ToMutate};
/// use syn::{visit_mut::VisitMut, File};
///
/// #[local_mutator]
/// #[derive(Debug, Clone)]
/// struct LocalImplementor<T: std::fmt::Debug + Clone>{
///   mutated: [bool; 4],
///   something: T 
/// }
///
/// impl<T> VisitMut for LocalImplementor<T> where T: std::fmt::Debug + Clone{
///   fn visit_file_mut(&mut self, _file: &mut File){
///     // Dummy implementation
///     self.mutated = [true; 4];
///   }
/// }
///
/// // The new implementor behaves as two `ItemToTrait` implementors + our local implementor.
/// #[mutator(ItemToTrait<'a>, ItemToTrait<'a>, local = LocalImplementor<T: std::fmt::Debug + Clone>)]
/// struct NewImplementor;
/// ```
///
/// Awesome
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
