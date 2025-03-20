// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::local_mutator;

#[local_mutator]
struct SomeStruct {
	mutated: u8,
}

fn main() {}
