// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::mutator::ToMutate;
use rust_writer_procedural::mutator;
use syn::visit_mut::VisitMut;

#[derive(Debug, Clone)]
struct SomeImplementor {
	mutated: [bool; 1],
}

#[mutator(local = SomeImplementor, rust_writer::ast::implementors::ItemToImpl<'a>)]
struct SomeStruct;

fn main() {}
