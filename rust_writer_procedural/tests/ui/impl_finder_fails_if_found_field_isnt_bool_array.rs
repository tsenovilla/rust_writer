// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_finder;

#[impl_finder]
struct SomeStruct {
	found: [u8; 1],
}

fn main() {}
