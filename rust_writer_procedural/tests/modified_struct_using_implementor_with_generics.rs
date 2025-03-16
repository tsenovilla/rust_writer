// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::{Finder, ToFind},
	implementors::{ItemToImpl, ItemToTrait},
	mutator::{Mutator, ToMutate},
};

use rust_writer_procedural::{finder, impl_finder, mutator};
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[impl_finder('a)]
#[derive(Debug, Clone)]
struct ToyFinderImplementor<T: std::fmt::Debug + Clone, const N: usize> {
	found: [bool; N],
	#[allow(dead_code)]
	something: T,
}

impl<'a, T: std::fmt::Debug + Clone, const N: usize> Visit<'a> for ToyFinderImplementor<T, N> {
	fn visit_file(&mut self, _file: &'a syn::File) {
		// Just say everything's found
		self.found = [true; N];
	}
}

#[finder(ItemToImpl<'a>, ItemToTrait<'a>, local = ToyFinderImplementor<T: std::fmt::Debug + Clone, N>)]
#[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
#[impl_from]
struct SomeStruct<const N: usize>;

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

		let toy_finder_implementor = ToyFinderImplementor { found: [false; 7], something: 0u8 };

		let some_struct: SomeStruct<7, u8> =
			(item_to_impl, item_to_trait, toy_finder_implementor).into();

		let ast = builder.get_mut_ast_file("trait_and_impl_block.rs").expect("This should exist");

		let mut finder: SomeStructFinderWrapper<7, u8> =
			Finder::default().to_find(&some_struct).into();
		assert!(!finder.find(ast));

		let mut mutator: SomeStructMutatorWrapper<7, u8> =
			Mutator::default().to_mutate(&some_struct).into();
		assert!(mutator.mutate(ast).is_ok());

		let mut finder: SomeStructFinderWrapper<7, u8> =
			Finder::default().to_find(&some_struct).into();
		assert!(finder.find(ast));
	});
}
