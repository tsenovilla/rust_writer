// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::mutator;

#[mutator(a, b, c)]
enum SomeEnum {
	A,
	B,
	C,
}

fn main() {}
