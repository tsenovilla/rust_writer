// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_finder;

#[impl_finder('a)]
enum SomeStruct {
	A,
	B,
	C,
}

fn main() {}
