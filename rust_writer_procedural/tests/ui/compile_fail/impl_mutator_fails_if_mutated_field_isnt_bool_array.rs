// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_mutator;

#[impl_mutator]
struct SomeStruct {
	mutated: [u8; 1],
}

fn main() {}
