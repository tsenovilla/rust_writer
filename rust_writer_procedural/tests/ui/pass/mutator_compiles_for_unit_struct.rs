// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	implementors::{ItemToImpl, ItemToTrait},
	mutator::ToMutate,
};
use rust_writer_procedural::mutator;
use syn::{parse_quote, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[mutator(ItemToTrait<'a>,ItemToImpl<'a>)]
#[impl_from]
struct SomeStruct;

fn main() {
	TestBuilder::default().with_trait_and_impl_block_ast().execute(|mut builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type2: From<String>;})).into();

		let item_to_impl: ItemToImpl = (
			"SomeTrait",
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn other_func(&self) -> bool{
						false
					 }
			  }),
		)
			.into();

		let some_struct: SomeStruct = (item_to_trait, item_to_impl).into();

		let ast = builder.get_mut_ast_file("trait_and_impl_block.rs").expect("This should exist");
	});
}
