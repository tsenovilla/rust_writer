// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::{
	ast::{
		finder::{Finder, ToFind},
		mutator::{Mutator, ToMutate},
	},
	test_builder::TestBuilder,
	Error,
};
use syn::parse_quote;

#[test]
fn item_to_impl_finder_find_item_if_present() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl: ItemToImpl = (
			"SomeTrait",
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_impl_finder_cannot_find_item_if_trait_name_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl: ItemToImpl = (
			"SomTrait",
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_finder_cannot_find_item_if_implementor_name_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl: ItemToImpl = (
			"SomeTrait",
			"SomImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_finder_cannot_find_item_if_item_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl: ItemToImpl = (
			"SomeTrait",
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn som_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_mutate_works() {
	TestBuilder::default().with_impl_block_ast().execute(|mut builder| {
		let item_to_impl: ItemToImpl = (
			"SomeTrait",
			"SomeImplementor",
			ImplItem::Type(parse_quote! {type Something = String;}),
		)
			.into();

		let ast = builder.get_mut_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(!finder.find(ast));

		let mutator = Mutator::default().to_mutate(&item_to_impl);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_impl_mutate_fails_if_cannot_find_impl_block() {
	TestBuilder::default().with_impl_block_ast().execute(|mut builder| {
		let item_to_impl: ItemToImpl = (
			"SomTrait",
			"SomeImplementor",
			ImplItem::Type(parse_quote! {type Something = String;}),
		)
			.into();

		let ast = builder.get_mut_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(!finder.find(ast));

		let mutator = Mutator::default().to_mutate(&item_to_impl);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
			if msg == format!("Cannot mutate using Mutator: {:?}", item_to_impl)
		));

		let mut finder = Finder::default().to_find(&item_to_impl);
		assert!(!finder.find(ast));
	});
}
