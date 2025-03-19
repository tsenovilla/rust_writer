// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::ToFind,
	implementors::{ItemToImpl, ItemToTrait},
};
use rust_writer_procedural::{finder,  mutator};
use syn::{visit::Visit, visit_mut::VisitMut};

#[finder(ItemToImpl<'a>, ItemToTrait<'a>)]
#[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
#[impl_from]
struct SomeStruct;

fn main() {}
