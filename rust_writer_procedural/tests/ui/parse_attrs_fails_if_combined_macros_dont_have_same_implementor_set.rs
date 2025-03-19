// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	mutator::ToMutate,
	implementors::{ItemToImpl, ItemToTrait},
};
use rust_writer_procedural::{finder,  mutator};
use syn::visit_mut::VisitMut;

#[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
#[finder(ItemToImpl<'a>, ItemToTrait<'a>)]
#[impl_from]
struct SomeStruct;

fn main() {}
