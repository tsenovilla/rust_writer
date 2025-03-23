// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	implementors::{ItemToImpl, ItemToTrait},
	mutator::ToMutate,
};
use rust_writer_proc::mutator;
use syn::{parse_quote, visit_mut::VisitMut, ImplItem, TraitItem};

#[mutator(ItemToTrait<'a>,ItemToImpl<'a>)]
struct SomeStruct;

fn main() {
	let item_to_trait: ItemToTrait =
		("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();

	let item_to_impl: ItemToImpl = (
		Some("SomeTrait"),
		"SomeImplementor",
		ImplItem::Fn(parse_quote! {
		fn some_func(&self) -> bool{
					true
				 }
		  }),
	)
		.into();

	let _some_struct: SomeStruct = (item_to_trait, item_to_impl).into();
}
