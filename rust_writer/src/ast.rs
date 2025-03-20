// SPDX-License-Identifier: GPL-3.0

//! # Description
//!
//! This module leverages the [syn](https://docs.rs/syn/latest/syn/index.html) crate and provides an easy mechanism
//! to interact with a rust AST represented as a [`syn::File`](https://docs.rs/syn/latest/syn/struct.File.html) with
//! just a few lines of code.
//!
//! At the heart of this module are the [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html)
//! and [`Mutator`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html)
//! structs, which can be thought of a game console:
//! - Their purpose is clear: they either find something in an AST or mutate parts of the AST itself
//!   (similar to how a game console's purpose is to run games).
//! - They can be customized to find or mutate specific parts of the AST in concrete ways (just as
//!   each game has its own unique narrative).
//!
//! These structs "load" the **implementors** (the games, following the previous analogy), which
//! specify exactly where and how the structs should act. For example, they may be used to locate a
//! specific item within a particular trait or to mutate a macro invocation, as the following
//! example shows:
//!
//! ```rust
//! use test_builder::TestBuilder;
//! use rust_writer::ast::{
//!   mutator::{Mutator, ToMutate},
//!   finder::{Finder, ToFind},
//!   implementors::{ItemToTrait, TokenStreamToMacro}
//! };
//! use syn::{
//!   visit::Visit,
//!   visit_mut::VisitMut,
//!   parse_quote,
//!   TraitItem
//! };
//!
//! TestBuilder::default()
//!  .with_trait_ast()
//!  .with_macro_ast()
//!  .execute(|mut builder|{
//!   // Define the ItemToTrait implementor. This implementor means:
//!   // 1. We're looking inside a trait called `MyTrait`   .
//!   // 2. We're interested in a type called `Type1` with trait bound `From<String>`.
//!   let item_to_trait: ItemToTrait =
//!    ("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();
//!
//!   let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");
//!
//!   // 'Load' the implementor into a `Finder` using `to_find`.
//!   let mut finder = Finder::default().to_find(&item_to_trait);
//!
//!   // Use the new finder variable to check if there's a trait called `MyTrait` in the AST
//!   // containing a type called `Type1` whose trait bound is `From<String>`.
//!   assert!(finder.find(ast));
//!
//!   // Define the TokenStreamToMacro implementor. This implementor means:
//!   // 1. We're looking inside a macro called `my_macro`.
//!   // 2. We're interested in a `TokenStream` composed by the token `D`.
//!   let token_to_macro: TokenStreamToMacro =
//!     (parse_quote! { my_macro }, None, parse_quote! { D }).into();
//!
//!   let ast = builder.get_mut_ast_file("macro.rs").expect("This exists; qed;");
//!
//!   let mut finder = Finder::default().to_find(&token_to_macro);
//!
//!   // The token `D` isn't in the macro at the beginning.
//!   assert!(!finder.find(ast));
//!
//!   // 'Load' the implementor into a 'Mutator' using `to_mutate` and mutate the ast.
//!   let mut mutator = Mutator::default().to_mutate(&token_to_macro);
//!   assert!(mutator.mutate(ast).is_ok());
//!
//!   // We can now find the token `D` inside `my_macro`.
//!   let mut finder = Finder::default().to_find(&token_to_macro);
//!   assert!(finder.find(ast));
//! });
//! ```
//!
//! This crate comes with a set of predefined implementors, which can be found in the
//! [implementors](https://docs.rs/rust_writer/latest/rust_writer/ast/implementors/index.html) module.
//! This set may be extended in the future to include additional implementors as the crate evolves.
//! All the included implementors comes with a `From` implementation that
//!
//! # Combining implementors
//!
//! It was so good up to this point, but creating a new `Finder`/`Mutator` for each AST operation
//! may feel a bit cumbersome in some situations. The 
//! [`#[finder]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.finder.html) and 
//! [`#[mutator]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.mutator.html) macros come in to combine several
//! implementors into a new one capable of executing all operations simultaneously.
//!
//! It's recomended to go through their docs to fully understand what those macros are doing and
//! how to use them. As a teaser, let's replicate the example of the item added to a trait and the
//! token stream added to a macro using this handy approach.
//!
//! ```rust
//! use test_builder::TestBuilder;
//! use rust_writer::ast::{
//!   mutator::{Mutator, ToMutate},
//!   finder::{Finder, ToFind},
//!   implementors::{ItemToTrait, TokenStreamToMacro},
//!   mutator,
//!   finder
//! };
//! use syn::{
//!   visit::Visit,
//!   visit_mut::VisitMut,
//!   parse_quote,
//!   TraitItem
//! };
//!
//! #[finder(ItemToTrait<'a>, TokenStreamToMacro)]
//! #[mutator(ItemToTrait<'a>, TokenStreamToMacro)]
//! #[impl_from]
//! struct CombinedImplementor;
//!
//! TestBuilder::default()
//!  .with_trait_ast()
//!  .with_macro_ast()
//!  .execute(|mut builder|{
//!   let item_to_trait: ItemToTrait =
//!    ("MyTrait", TraitItem::Type(parse_quote! { type Type1: From<String>; })).into();
//!
//!   let token_to_macro: TokenStreamToMacro =
//!     (parse_quote! { my_macro }, None, parse_quote! { D }).into();
//!   
//!   let combined_implementor: CombinedImplementor = (item_to_trait, token_to_macro).into();
//!
//!   let mut ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;").clone();
//!   ast.items.extend(builder.get_ref_ast_file("macro.rs").expect("This exists;
//!   qed;").items.clone());
//!
//!   let mut finder: CombinedImplementorFinderWrapper = Finder::default().to_find(&combined_implementor).into();
//!
//!   // The `Finder` fails to find all the elements.
//!   assert!(!finder.find(&ast, None));
//!
//!   // But we can still check that the item_to_trait implementor succeeded in its research,
//!   // thanks to the finder struct and the handy `get_missing_indexes` method that tell us which
//!   // implementors couldn't find its target.
//!   let missing_indexes = finder.get_missing_indexes();
//!   assert_eq!(missing_indexes, Some(vec![1]));
//!
//!   // And that it was the macro implementor who failed
//!   assert!(!finder.0.found[1]);
//!
//!   // Let's mutate and complete the ast
//!   let mut mutator: CombinedImplementorMutatorWrapper = Mutator::default().to_mutate(&combined_implementor).into();
//!
//!   // We can mutate just the elements that were not found, so we don't duplicate the rest.
//!   assert!(mutator.mutate(&mut ast, missing_indexes.as_deref()).is_ok());
//!
//!   let mut finder: CombinedImplementorFinderWrapper = Finder::default().to_find(&combined_implementor).into();
//!
//!   // Now the finder can find both elements
//!   assert!(finder.find(&ast, None));
//! });
//! ```
//! # Defining new implementors
//!
//! If the set of predefined implementors isn't enough, defining a new implementor is perfectly
//! possible. However, it's not directly feasible... Both `Finder` and
//! `Mutator` need to implement [`syn::visit::Visit`](https://docs.rs/syn/latest/syn/visit/trait.Visit.html)
//! and [`syn::visit_mut::VisitMut`](https://docs.rs/syn/latest/syn/visit_mut/trait.VisitMut.html) traits respectively
//! in order to become functional. Implementing a foreign trait in a foreign type is forbidden by
//! the orphan rule, so, what to do in this case? 🤔🤔🤔
//!
//! There's basically two approaches:
//! 1. If the implementor to define isn't quite linked to the project where it's needed, open a **PR**
//!    to the rust_writer crate! That way everybody will benefit of the new implementor.
//! 1. Use the [`#[impl_finder]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.impl_finder.html)
//!    and [`#[impl_mutator]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.impl_mutator.html)
//!    macros to define a local implementor.

pub mod finder;
pub mod implementors;
mod macros;
pub mod mutator;

pub use macros::*;
