// SPDX-License-Identifier: GPL-3.0

//! This module contains the [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html)
//! struct, which is used to search for specific items in an AST in a very targeted way.
//! The `Finder` struct is completely generic, and thanks to the
//! [`ToFind`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/trait.ToFind.html) trait and the
//! [implementors](https://docs.rs/rust_writer/latest/rust_writer/ast/implementors/index.html) it
//! may be customized to assert if an element is contained inside an AST.
//!
//! # Example
//! ```rust
//! use test_builder::TestBuilder;
//! use rust_writer::ast::{
//!     finder::{Finder, ToFind},
//!     implementors::ItemToTrait
//! };
//! use syn::{
//!     visit::Visit,
//!     parse_quote,
//!     File
//! };
//!
//! TestBuilder::default()
//!     .with_trait_ast()
//!     .execute(|builder| {
//!         let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");
//!
//!         // Define the ItemFinder implementor. This implementor specifies:
//!         // 1. The trait we want to search for, e.g., a trait named `MyTrait`.
//!         // 2. The specific item(s) we expect to find within that trait.
//!         let item_to_trait: ItemToTrait =
//!             ("MyTrait", parse_quote! { type Type1: From<String>; }).into();
//!
//!         // Create the Finder and load this particular implementor
//!         let mut finder = Finder::default().to_find(&item_to_trait);
//!
//!         // Search the AST for the expected item
//!         assert!(finder.find(&ast));
//!     });
//! ```

#[cfg(test)]
mod tests;

use std::fmt::Debug;
use syn::{visit::Visit, File};

/// A placeholder finder which does not perform any specific search.
/// This is used as the default for a [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html).
#[derive(Debug)]
pub struct EmptyFinder;

/// The `Finder` struct is used to assert if a concrete element belongs to an AST.
/// The `Finder` struct is totally generic, so it can be thought of a game console: its purpose is
/// to determine if an element is contained inside an AST but it doesn't know how to do it until an
/// [implementor](https://docs.rs/rust_writer/latest/rust_writer/ast/implementors/index.html)
/// is loaded. This implementor will tell `Finder` how it should behave.
///
/// A new `Finder` can be easily created using `Finder::default()`, which returns a
/// `Finder<'_, EmptyFinder, 1>`. This `Finder` is useless up to this point, but thanks to the
/// [`ToFind`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/trait.ToFind.html)
/// trait, an implementor can be seamlessly loaded to the `Finder`, releasing its whole power.
///
/// Once configured, the [`find`](#method.find) method can be called to look inside the AST.
///
/// ```rust
/// use test_builder::TestBuilder;
/// use rust_writer::ast::{
///     finder::{Finder, ToFind},
///     implementors::ItemToTrait
/// };
/// use syn::{
///     visit::Visit,
///     parse_quote,
///     File
/// };
///
/// TestBuilder::default()
///     .with_trait_ast()
///     .execute(|builder| {
///         let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");
///
///         // Define the ItemFinder implementor. This implementor specifies:
///         // 1. The trait we want to search for, e.g., a trait named `MyTrait`.
///         // 2. The specific item(s) we expect to find within that trait.
///         let item_to_trait: ItemToTrait =
///             ("MyTrait", parse_quote! { type Type1: From<String>; }).into();
///
///         // Create the Finder and load this particular implementor
///         let mut finder = Finder::default().to_find(&item_to_trait);
///
///         // Search the AST for the expected item
///         assert!(finder.find(&ast));
///     });
/// ```
#[derive(Debug, Clone)]
pub struct Finder<'a, T: Debug, const N: usize> {
	/// A `Finder` can act in different parts of the AST at the same time. This array keeps track
	/// of which of these searchs succeeded.  
	pub found: [bool; N],
	/// Placeholder to load an implementor.
	pub finder: &'a T,
}

/// This trait is used to create a new `Finder` variable.
/// It is typically implemented for `Finder<'_, EmptyFinder, 1>`, enabling the flow:
/// 1. Create a `Finder<'_, EmptyFinder, 1>` using `Finder::default()`.
/// 2. Configure it with the desired implementor using the `to_find` method.
pub trait ToFind<'a, T: Debug, const N: usize> {
	fn to_find(self, finder: &'a T) -> Finder<'a, T, N>;
}

impl Default for Finder<'_, EmptyFinder, 1> {
	fn default() -> Self {
		Self { found: [false], finder: &EmptyFinder }
	}
}

impl<'a, T, const N: usize> Finder<'a, T, N>
where
	T: Debug,
	Finder<'a, T, N>: Visit<'a>,
{
	/// Apply all the searches defined by the implementor.
	pub fn find(&mut self, ast: &'a File) -> bool {
		self.visit_file(ast);
		self.found.iter().all(|&x| x)
	}
}

impl<T, const N: usize> Finder<'_, T, N>
where
	T: Debug,
{
	/// Reset the `found` array to all `false` values.
	pub fn reset(&mut self) {
		self.found = [false; N];
	}
}
