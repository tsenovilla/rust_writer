// SPDX-License-Identifier: GPL-3.0

use rust_writer::ast::{
	finder::ToFind,
	implementors::{ItemToImpl, ItemToTrait},
	mutator::ToMutate,
};

use rust_writer_procedural::{finder, impl_finder, mutator};
use syn::{parse_quote, visit::Visit, visit_mut::VisitMut, ImplItem, TraitItem};
use test_builder::TestBuilder;

#[impl_finder('a)]
#[derive(Debug, Clone)]
struct ToyFinderImplementor {
	found: [bool; 0],
}

impl<'a> Visit<'a> for ToyFinderImplementor {}

#[finder(ItemToImpl<'a>, ItemToTrait<'a>, local = ToyFinderImplementor)]
#[mutator(ItemToTrait<'a>, ItemToImpl<'a>)]
#[impl_from]
struct SomeStruct<T: std::fmt::Debug + Clone, const N: usize> {
	#[allow(dead_code)]
	some_useful_field: T,
	#[allow(dead_code)]
	some_useful_array: [u8; N],
}

#[test]
fn impl_from_with_outermost_macro_implementors_set_bigger_than_innermost_macro_implementors_set() {
	TestBuilder::default().with_trait_and_impl_block_ast().execute(|_| {
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

		let toy_finder_implementor = ToyFinderImplementor { found: [] };

		let _some_struct: SomeStruct<String, 20> = (
			"Hello world".to_owned(),
			[27; 20],
			item_to_impl,
			item_to_trait,
			toy_finder_implementor,
		)
			.into();
	});
}
