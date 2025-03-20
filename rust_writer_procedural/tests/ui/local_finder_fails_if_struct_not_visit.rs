// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::local_finder;

#[local_finder('a)]
#[derive(Debug)]
struct SomeStruct {
	found: [bool; 1],
}

fn main() {}
