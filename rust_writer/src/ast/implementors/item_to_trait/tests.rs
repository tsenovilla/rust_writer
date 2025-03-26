// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::Error;
use syn::parse_quote;
use test_builder::TestBuilder;

#[test]
fn item_to_trait_finder_find_item_if_present() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_trait_finder_find_item_if_present_despite_attrs() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type CommentedType: From<String>;})).into();

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_trait_finder_cannot_find_item_if_trait_name_incorrect() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let item_to_trait: ItemToTrait =
			("MTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_trait_finder_cannot_find_item_if_item_incorrect() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<u8>;})).into();

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_trait_mutate_works() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Something: From<String>;})).into();

		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_trait);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_trait_mutate_fails_if_cannot_find_trait() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let item_to_trait: ItemToTrait =
			("UnexistingTrait", TraitItem::Type(parse_quote! {type Something: From<String>;}))
				.into();

		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_trait);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
			if msg == format!("Cannot mutate using Mutator: {:?}", item_to_trait)
		));

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));
	});
}
