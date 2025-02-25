// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_mutator;

#[impl_mutator]
enum SomeStruct {
	A,
	B,
	C,
}

fn main() {}
