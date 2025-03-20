// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::local_finder;

#[local_finder('a)]
struct SomeStruct {
	found: [u8; 1],
}

fn main() {}
