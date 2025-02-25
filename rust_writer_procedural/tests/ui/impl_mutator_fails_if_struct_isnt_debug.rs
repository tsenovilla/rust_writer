// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_mutator;
use syn::visit_mut::VisitMut;

#[impl_mutator]
struct SomeStruct {
	mutated: [bool; 1],
}

impl VisitMut for SomeStruct {}

fn main() {}
