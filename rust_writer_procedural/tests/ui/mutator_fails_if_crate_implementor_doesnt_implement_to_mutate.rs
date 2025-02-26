// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	implementors::ItemToImpl,
	mutator::{EmptyMutator, ToMutate},
};
use rust_writer_procedural::mutator;
use syn::visit_mut::VisitMut;

#[mutator(ItemToImpl<'a>, EmptyMutator)]
struct SomeStruct;

fn main() {}
