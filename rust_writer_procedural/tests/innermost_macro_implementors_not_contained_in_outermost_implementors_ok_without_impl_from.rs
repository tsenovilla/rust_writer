// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::ToFind,
	implementors::{ItemToImpl, ItemToTrait},
	mutator::ToMutate,
};
use rust_writer_procedural::{finder, impl_finder, mutator};
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[impl_finder]
#[derive(Debug, Clone)]
struct ToyFinderImplementor {
	found: [bool; 0],
}

impl Visit<'_> for ToyFinderImplementor {}

#[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
#[finder(ItemToImpl<'a>, ItemToTrait<'a>, local = ToyFinderImplementor)]
struct SomeStruct;

#[test]
fn inner_most_macro_implementors_not_contained_in_outermost_implementors_ok_without_impl_from() {
	TestBuilder::default().with_trait_and_impl_block_ast().execute(|_| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type3: From<String>;})).into();

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

		let toy_finder_implementor = ToyFinderImplementor { found: [] };

		let _some_struct = SomeStruct {
			itemtoimpl: item_to_impl,
			itemtotrait: item_to_trait,
			toyfinderimplementor: toy_finder_implementor,
		};
	});
}
