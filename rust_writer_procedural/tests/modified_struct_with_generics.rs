// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::{Finder, ToFind},
	implementors::{ItemToImpl, ItemToTrait},
	mutator::{Mutator, ToMutate},
};

use rust_writer_procedural::{finder, mutator};
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
#[finder(ItemToImpl<'a>, ItemToTrait<'a>)]
#[impl_from]
struct SomeStruct<T: Clone + std::fmt::Debug + From<String>>{
    some_super_useful_field: T
}

#[test]
fn modified_struct_with_generics() {
	TestBuilder::default().with_trait_and_impl_block_ast().execute(|mut builder| {
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

		let some_struct: SomeStruct<Vec<u8>> = (vec![1,2,3],item_to_trait, item_to_impl ).into();

		let ast = builder.get_mut_ast_file("trait_and_impl_block.rs").expect("This should exist");

		let mut finder: SomeStructFinderWrapper<Vec<u8>> = Finder::default().to_find(&some_struct).into();
		assert!(!finder.find(ast));

		let mut mutator: SomeStructMutatorWrapper<Vec<u8>> =
			Mutator::default().to_mutate(&some_struct).into();
		assert!(mutator.mutate(ast).is_ok());

		let mut finder: SomeStructFinderWrapper<Vec<u8>> = Finder::default().to_find(&some_struct).into();
		assert!(finder.find(ast));

    assert_eq!(some_struct.some_super_useful_field, vec![1,2,3]);
	});
}
