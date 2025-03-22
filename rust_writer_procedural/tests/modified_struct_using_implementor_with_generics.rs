// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::{Finder, ToFind},
	implementors::{ItemToImpl, ItemToTrait},
	mutator::{Mutator, ToMutate},
};

use rust_writer_procedural::{finder, local_finder, local_mutator, mutator};
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[local_finder('a)]
#[local_mutator]
#[derive(Debug, Clone)]
struct ToyImplementor<T: std::fmt::Debug + Clone, const N: usize> {
	found: [bool; N],
	mutated: [bool; N],
	#[allow(dead_code)]
	something: T,
}

impl<'a, T: std::fmt::Debug + Clone, const N: usize> Visit<'a> for ToyImplementor<T, N> {
	fn visit_file(&mut self, _file: &'a syn::File) {
		// Just say everything's found
		self.found = [true; N];
	}
}

impl<T: std::fmt::Debug + Clone, const N: usize> VisitMut for ToyImplementor<T, N> {
	fn visit_file_mut(&mut self, _file: &mut syn::File) {
		// Just say everything's found
		self.mutated = [true; N];
	}
}

#[finder(ItemToImpl<'a>, ItemToTrait<'a>, local = ToyImplementor<T,N>)]
#[mutator(ItemToImpl<'a>, ItemToTrait<'a>, local = ToyImplementor<T,N>)]
#[impl_from]
struct SomeStruct<T: std::fmt::Debug + Clone, const N: usize> {
	#[allow(dead_code)]
	some_useful_array: [T; N],
}

#[test]
fn modified_struct_using_local_implementor_with_generics() {
	TestBuilder::default().with_trait_and_impl_block_ast().execute(|mut builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type3: From<String>;})).into();

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

		let toy_finder_implementor =
			ToyImplementor { found: [false; 7], mutated: [false; 7], something: 0u8 };

		let some_struct: SomeStruct<u8, 7> =
			([0; 7], item_to_impl, item_to_trait, toy_finder_implementor).into();

		let ast = builder.get_mut_ast_file("trait_and_impl_block.rs").expect("This should exist");

		let mut finder: SomeStructFinderWrapper<u8, 7> =
			Finder::default().to_find(&some_struct).into();
		assert!(!finder.find(ast, None));

		let mut mutator: SomeStructMutatorWrapper<u8, 7> =
			Mutator::default().to_mutate(&some_struct).into();
		assert!(mutator.mutate(ast, None).is_ok());

		let mut finder: SomeStructFinderWrapper<u8, 7> =
			Finder::default().to_find(&some_struct).into();
		assert!(finder.find(ast, None));
	});
}
