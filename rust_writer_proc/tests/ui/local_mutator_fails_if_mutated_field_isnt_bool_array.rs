// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::local_mutator;

#[local_mutator]
struct SomeStruct {
	mutated: [u8; 1],
}

fn main() {}
