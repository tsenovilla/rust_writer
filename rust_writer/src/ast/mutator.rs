// SPDX-License-Identifier: GPL-3.0

//! This module contains the [`Mutator`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html)
//! struct, which is used to mutate an AST in a very targeted way. The `Mutator` struct is
//! totally generic, and thanks to the [`ToMutate`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/trait.ToMutate.html)
//! trait and the [implementors](https://docs.rs/rust_writer/latest/rust_writer/ast/implementors/index.html),
//! it may be customized to mutate the AST in pretty different ways.
//!
//! # Example
//! ```rust
//! use test_builder::TestBuilder;
//! use rust_writer::ast::{
//!   mutator::{Mutator, ToMutate},
//!   implementors::ItemToTrait
//! };
//! use syn::{
//!   visit_mut::VisitMut,
//!   parse_quote,
//!   TraitItem
//! };
//!
//! TestBuilder::default()
//!  .with_trait_ast()
//!  .execute(|mut builder|{
//!   let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");
//!
//!   // Define the ItemToTrait implementor. This implementor means:
//!   // 1. We're looking inside a trait called `MyTrait`   .
//!   // 2. We're interested in adding a type called `Type3` with trait bound `From<String>` to
//!   //    that trait.
//!   let item_to_trait: ItemToTrait =
//!    ("MyTrait", parse_quote! {type Type3: From<String>;}).into();
//!
//!   // Create the Mutator and load this particular implementor
//!   let mut mutator = Mutator::default().to_mutate(&item_to_trait);
//!
//!   // Mutate the AST
//!   assert!(mutator.mutate(ast).is_ok());
//! });
//! ```

#[cfg(test)]
mod tests;

use crate::Error;
use std::fmt::Debug;
use syn::{visit_mut::VisitMut, File};

/// A placeholder finder which does not perform any specific search.
/// This is used as the default for a [`Mutator`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html) .
#[derive(Debug, Clone)]
pub struct EmptyMutator;

/// The `Mutator` struct is used to mutate an AST in very targeted way.
/// The `Mutator` struct is totally generic, so it can be thought of a game console: its purpose is
/// to mutate an AST but it doesn't know how to do it until an
/// [implementor](https://docs.rs/rust_writer/latest/rust_writer/ast/implementors/index.html)
/// is loaded. This implementor will tell `Mutator` how it should behave.
///
/// A new `Mutator` can be easily created using `Mutator::default()`, which returns a
/// `Mutator<'_, EmptyMutator, 1>`. This `Mutator` is useless up to this point, but thanks to the
/// [`ToMutate`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/trait.ToMutate.html)
/// trait, an implementor can be seamlessly loaded to the `Mutator`, releasing its whole power.
///
/// Once configured, the [`mutate`](#method.mutate) method can be called to mutate the AST.
///
/// ```rust
/// use test_builder::TestBuilder;
/// use rust_writer::ast::{
///   mutator::{Mutator, ToMutate},
///   implementors::ItemToTrait
/// };
/// use syn::{
///   visit_mut::VisitMut,
///   parse_quote,
///   TraitItem
/// };
///
/// TestBuilder::default()
///  .with_trait_ast()
///  .execute(|mut builder|{
///   let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");
///
///   // Define the ItemToTrait implementor. This implementor means:
///   // 1. We're looking inside a trait called `MyTrait`   .
///   // 2. We're interested in adding a type called `Type3` with trait bound `From<String>` to
///   //    that trait.
///   let item_to_trait: ItemToTrait =
///    ("MyTrait", parse_quote! {type Type3: From<String>;}).into();
///
///   // Create the Mutator and load this particular implementor
///   let mut mutator = Mutator::default().to_mutate(&item_to_trait);
///
///   // Mutate the AST
///   assert!(mutator.mutate(ast).is_ok());
/// });
/// ```
#[derive(Debug, Clone)]
pub struct Mutator<'a, T: Debug + Clone, const N: usize> {
	/// A `Mutator` can act in different parts of the AST at the same time. This array keeps track
	/// of which of these mutations succeeded.  
	pub mutated: [bool; N],
	/// Placeholder to load an implementor.
	pub mutator: &'a T,
}

/// This trait is used to create new `Mutator` variables. It's typically implemented for
/// `Mutator<'_, EmptyMutator, 1>`, enabling the flow:
/// 1. Create a `Mutator<'_, EmptyMutator, 1>` using `Mutator::default()`.
/// 1. Load the needed implementor using `to_mutate`.
pub trait ToMutate<'a, T: Debug + Clone, const N: usize> {
	fn to_mutate(self, mutator: &'a T) -> Mutator<'a, T, N>;
}

impl Default for Mutator<'_, EmptyMutator, 1> {
	fn default() -> Self {
		Self { mutated: [false], mutator: &EmptyMutator }
	}
}

impl<'a, T, const N: usize> Mutator<'a, T, N>
where
	T: Debug + Clone,
	Mutator<'a, T, N>: VisitMut,
{
	/// Apply all the mutations defined by the implementor.
	pub fn mutate(&mut self, ast: &mut File) -> Result<(), Error> {
		self.visit_file_mut(ast);

		if self.mutated.iter().all(|&x| x) {
			Ok(())
		} else {
			Err(Error::Descriptive(format!("Cannot mutate using Mutator: {:?}", self.mutator)))
		}
	}
}

impl<T, const N: usize> Mutator<'_, T, N>
where
	T: Debug + Clone,
{
	/// Reset the `mutated` array to all `false` values.
	pub fn reset(&mut self) {
		self.mutated = [false; N];
	}
}
