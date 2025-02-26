// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_finder;

#[impl_finder]
#[derive(Debug)]
struct SomeStruct {
	found: [bool; 1],
}

fn main() {}
