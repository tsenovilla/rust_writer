// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::Error;
use syn::parse_quote;
use test_builder::TestBuilder;

#[test]
fn item_to_mod_finder_find_item_if_present() {
	TestBuilder::default().with_mod_ast().execute(|builder| {
		let item_to_mod: ItemToMod =
			("SomeMod", parse_quote! { fn some_super_func(&self) -> bool { true } }).into();

		let ast = builder.get_ref_ast_file("mod.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_mod_finder_find_item_if_present_despite_docs() {
	TestBuilder::default().with_mod_ast().execute(|builder| {
		let item_to_mod: ItemToMod = ("SomeMod", parse_quote! { trait SomeTrait{} }).into();

		let ast = builder.get_ref_ast_file("mod.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_mod_finder_cannot_find_item_if_mod_name_incorrect() {
	TestBuilder::default().with_mod_ast().execute(|builder| {
		let item_to_mod: ItemToMod =
			("OtherMod", parse_quote! { fn some_super_func(&self) -> bool { true } }).into();

		let ast = builder.get_ref_ast_file("mod.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_mod_finder_cannot_find_item_if_item_incorrect() {
	TestBuilder::default().with_mod_ast().execute(|builder| {
		let item_to_mod: ItemToMod =
			("SomeMod", parse_quote! { fn some_super_func(&self) -> bool { false } }).into();

		let ast = builder.get_ref_ast_file("mod.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_mod_mutate_works() {
	TestBuilder::default().with_mod_ast().execute(|mut builder| {
		let item_to_mod: ItemToMod =
			("SomeMod", parse_quote! { fn new_func() -> i32 { 42 } }).into();

		let ast = builder.get_mut_ast_file("mod.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_mod);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_mod_mutate_fails_if_cannot_find_mod() {
	TestBuilder::default().with_mod_ast().execute(|mut builder| {
		let item_to_mod: ItemToMod =
			("NonExistingMod", parse_quote! { fn new_func() -> i32 { 42 } }).into();

		let ast = builder.get_mut_ast_file("mod.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_mod);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
			if msg == format!("Cannot mutate using Mutator: {:?}", item_to_mod)
		));

		let mut finder = Finder::default().to_find(&item_to_mod);
		assert!(!finder.find(ast));
	});
}
