// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	implementors::{ItemToImpl},
	finder::{ToFind},
    mutator::EmptyMutator
};
use rust_writer_procedural::{finder};
use syn::{visit::Visit};

// Use EmptyMutator as it implements Clone, so the only fail here is that ToFind isn't implemented
#[finder(ItemToImpl<'a>, EmptyMutator)]
struct SomeStruct;

fn main(){}
