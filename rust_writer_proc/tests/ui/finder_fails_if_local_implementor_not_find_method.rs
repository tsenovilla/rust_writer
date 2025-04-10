// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::finder::ToFind;
use rust_writer_proc::finder;

// A custom mutator emulating ItemToTrait
#[derive(Debug, Clone)]
struct SomeImplementor {
	found: [bool; 1],
}

#[finder(local = SomeImplementor, rust_writer::ast::implementors::ItemToImpl<'a>)]
struct SomeStruct;

fn main() {}
