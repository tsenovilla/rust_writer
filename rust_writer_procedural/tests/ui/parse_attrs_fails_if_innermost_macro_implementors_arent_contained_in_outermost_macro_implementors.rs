// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::ToFind,
	implementors::{ItemToImpl, ItemToTrait},
};
use rust_writer_procedural::{finder, impl_mutator, mutator};
use syn::{visit::Visit, visit_mut::VisitMut};

#[impl_mutator]
#[derive(Debug)]
struct SomeImplementor {
	mutated: [bool; 1],
}

impl VisitMut for SomeImplementor {
	fn visit_item_trait_mut(&mut self, _item_trait: &mut syn::ItemTrait) {}
}

#[finder(ItemToImpl<'a>, ItemToTrait<'a>)]
#[mutator(ItemToTrait<'a>, local = SomeImplementor, ItemToImpl<'a>)]
#[impl_from]
struct SomeStruct;

fn main() {}
