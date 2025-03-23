// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::local_mutator;

#[local_mutator]
enum SomeStruct {
	A,
	B,
	C,
}

fn main() {}
