// SPDX-License-Identifier: GPL-3.0

//! This module contains a predefined set of implementors that can be used with
//! [`Finder`](https://docs.rs/rust_writer/latest/rust_writer/ast/finder/struct.Finder.html) and
//! [`Mutator`](https://docs.rs/rust_writer/latest/rust_writer/ast/mutator/struct.Mutator.html) to
//! effectively interact with an AST.
//!
//! The [`#[local_finder]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.local_finder.html)
//! and
//! [`#[local_mutator]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.local_mutator.html)
//! macros are useful to define new implementors out of this crate, while the
//! [`#[finder]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.finder.html) and
//! [`#[mutator]`](https://docs.rs/rust_writer/latest/rust_writer/ast/attr.mutator.html) macros
//! come in handy to create a new implementor merging other implementors functionalities.
//!
//! # Disclaimer
//!
//! The set of implementors is still limited and will be updated as needed. Any PR to
//! [the repo](https://github.com/tsenovilla/rust_writer) introducing new implementors is more than
//! welcome.

mod item_to_file;
mod item_to_impl;
mod item_to_mod;
mod item_to_trait;
mod token_stream_to_macro;

pub use item_to_file::ItemToFile;
pub use item_to_impl::ItemToImpl;
pub use item_to_mod::ItemToMod;
pub use item_to_trait::ItemToTrait;
pub use token_stream_to_macro::TokenStreamToMacro;
