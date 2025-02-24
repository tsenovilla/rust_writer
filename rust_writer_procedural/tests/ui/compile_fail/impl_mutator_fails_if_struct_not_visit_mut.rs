// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_mutator;

#[impl_mutator]
#[derive(Debug)]
struct SomeStruct {
	mutated: [bool; 1],
}

fn main() {}
