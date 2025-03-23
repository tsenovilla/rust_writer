// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::local_mutator;
use syn::visit_mut::VisitMut;

#[local_mutator]
struct SomeStruct {
	mutated: [bool; 1],
}

impl VisitMut for SomeStruct {}

fn main() {}
