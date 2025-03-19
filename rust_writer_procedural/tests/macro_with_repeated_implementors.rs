// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::{Finder, ToFind},
	implementors::{ItemToImpl, ItemToTrait},
	mutator::{Mutator, ToMutate},
};
use rust_writer_procedural::{finder, mutator};
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[mutator(ItemToTrait<'a>, ItemToTrait<'a>, ItemToImpl<'a>)]
#[finder(ItemToTrait<'a>, ItemToTrait<'a>, ItemToImpl<'a>)]
#[impl_from]
struct SomeStruct;

#[test]
fn modified_unit_struct() {
	TestBuilder::default().with_trait_and_impl_block_ast().execute(|mut builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type3: From<String>;})).into();

		let item_to_trait2: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type4: From<String>;})).into();

		let item_to_impl: ItemToImpl = (
			Some("SomeTrait"),
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn other_func(&self) -> bool{
						false
					 }
			  }),
		)
			.into();

		let some_struct: SomeStruct = (item_to_trait, item_to_trait2, item_to_impl).into();

		let ast = builder.get_mut_ast_file("trait_and_impl_block.rs").expect("This should exist");

		let mut finder: SomeStructFinderWrapper = Finder::default().to_find(&some_struct).into();
		assert!(!finder.find(ast));

		let mut mutator: SomeStructMutatorWrapper =
			Mutator::default().to_mutate(&some_struct).into();
		assert!(mutator.mutate(ast).is_ok());

		let mut finder: SomeStructFinderWrapper = Finder::default().to_find(&some_struct).into();
		assert!(finder.find(ast));
	});
}
