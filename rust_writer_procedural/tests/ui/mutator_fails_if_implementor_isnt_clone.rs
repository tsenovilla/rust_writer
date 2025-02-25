// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::mutator::ToMutate;
use rust_writer_procedural::{impl_mutator, mutator};
use syn::visit_mut::VisitMut;

#[impl_mutator]
#[derive(Debug)]
struct A {
	mutated: [bool; 1],
}

impl VisitMut for A {}

#[mutator(rust_writer::ast::implementors::ItemToTrait<'a>, local = A)]
struct SomeStruct;

fn main() {}
