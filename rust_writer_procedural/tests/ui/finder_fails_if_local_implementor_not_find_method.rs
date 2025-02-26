// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::{finder};
use syn::{visit::Visit};
use rust_writer::ast::finder::ToFind;

// A custom mutator emulating ItemToTrait
#[derive(Debug, Clone)]
struct SomeImplementor {
	found: [bool; 1],
}

#[finder(local = SomeImplementor, rust_writer::ast::implementors::ItemToImpl<'a>)]
struct SomeStruct;

fn main(){}
