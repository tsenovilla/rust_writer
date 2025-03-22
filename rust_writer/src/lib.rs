// SPDX-License-Identifier: GPL-3.0

//! # Description
//!
//! `rust_writer` is a crate designed to simplify meta programming in Rust—but in a different way than the typical procedural macro crates (such as [`syn`], [`quote`](https://docs.rs/quote/latest/quote/), and [`proc_macro2`]).
//!
//! While those crates excel at writing procedural macros, `rust_writer` leverages their
//! capabilities to modify Rust source files. This makes it ideal for tasks that require in-place
//! source code modifications rather than merely generating new code visible only to the compiler.
//!
//! The crate is divided into two modules: the [`preserver`] module and the [`ast`] module. Although
//! these modules can be used separately and even for purposes other than the crate's primary
//! objective, using them together unlocks the full potential of the crate.
//!
//! - The [`preserver`] module ensures that the original structure of the source code is maintained
//!   when it is parsed into an AST.
//! - The [`ast`] module provides various tools to simplify AST interactions, allowing precise
//!   modifications exactly where needed.
//!
//! For further details, please refer to the individual module documentation. A complete example is
//! often the best way to illustrate the functionality:
//!
//! ```rust
//! use quote::quote;
//! use rust_writer::{
//!     ast::{
//!         implementors::{ItemToFile, ItemToImpl, TokenStreamToMacro},
//!         mutator,
//!         mutator::{Mutator, ToMutate},
//!     },
//!     preserver::Preserver,
//! };
//! use syn::{parse_quote, ImplItem, Item, visit_mut::VisitMut};
//! use test_builder::TestBuilder;
//!
//! // A mutator defined with the given implementors.
//! #[mutator(TokenStreamToMacro, ItemToFile, ItemToImpl<'a>)]
//! #[impl_from]
//! struct TestMutator;
//!
//! TestBuilder::default()
//!     .with_complete_file()
//!     .with_expanded_file()
//!     .execute(|builder| {
//!         let complete_file_path = builder.tempfile_path("complete_file.rs")
//!             .expect("This file exists");
//!         let expanded_file_path = builder.tempfile_path("expanded_file.rs")
//!             .expect("This file exists");
//!         let expected_code = std::fs::read_to_string(&expanded_file_path)
//!             .expect("File should be readable");
//!
//!         // Preserve an impl block and the invocation of `my_macro` in the code.
//!         // The rest of the file remains unchanged, preserving its original AST structure.
//!         let preserver1 = Preserver::new("impl MyTrait for MyStruct");
//!         let mut preserver2 = Preserver::new("fn main");
//!         preserver2.add_inners(&["my_macro"]);
//!
//!         let mut ast = rust_writer::preserver::preserve_and_parse(
//!             complete_file_path,
//!             &[&preserver1, &preserver2],
//!         )
//!         .expect("Preservation should be applied");
//!
//!         // Add a function to the trait implementation.
//!         // Note the TEMP_DOC comment: it will become an empty line in the final code,
//!         // which is a neat trick to insert an empty line at the start of the function.
//!         let item_to_impl: ItemToImpl = (
//!             Some("MyTrait"),
//!             "MyStruct",
//!             ImplItem::Fn(parse_quote! {
//!                 ///TEMP_DOC
//!                 fn func(&self) -> bool {
//!                     false
//!                 }
//!             }),
//!         )
//!         .into();
//!
//!         // Add a TokenStream to the `my_macro` invocation.
//!         let token_stream_to_macro: TokenStreamToMacro = (
//!             parse_quote!(my_macro),
//!             None,
//!             quote! {
//!                 struct SomeStruct {
//!                     field: u8,
//!                     string: String
//!                 }
//!             },
//!         )
//!         .into();
//!
//!         // Insert an item into the file so that `Path` is in scope.
//!         let item: Item = parse_quote!( use std::path::Path; );
//!         let item_to_file: ItemToFile = item.into();
//!
//!         // Create a mutator for the given implementors.
//!         let test_mutator: TestMutator =
//!             (token_stream_to_macro, item_to_file, item_to_impl).into();
//!         let mut mutator: TestMutatorMutatorWrapper =
//!             Mutator::default().to_mutate(&test_mutator).into();
//!
//!         // Mutate the AST.
//!         assert!(mutator.mutate(&mut ast, None).is_ok());
//!
//!         // Unparse the AST and restore the preserved code.
//!         assert!(rust_writer::preserver::resolve_preserved(&ast, complete_file_path).is_ok());
//!
//!         let actual_code = std::fs::read_to_string(complete_file_path)
//!             .expect("File should be readable");
//!         assert_eq!(actual_code, expected_code);
//!     });
//! ```
//!
//! # Important Note
//!
//! To simplify testing of this crate, a crate called `test_builder` is included as a dev
//! dependency. This crate makes it easy to set up tests that load different files and ASTs
//! seamlessly, allowing tests to focus on the specific functionality being verified.
//!
//! This crate is widely used across the documentation, so it's worthy to mention that the sample
//! files used by `TestBuilder` are available in
//! [the repository](https://github.com/tsenovilla/rust_writer/tree/main/test_builder/sample_files) and can be consulted as needed.
//!
//! The syntax for `TestBuilder` is straightforward: methods like `with_something_ast` load an AST
//! from a file named accordingly, while `with_something_file` load the entire file.

pub mod ast;
mod error;
pub mod preserver;

pub use error::Error;
