// SPDX-License-Identifier: GPL-3.0

use super::*;
use syn::parse_quote;
use test_builder::TestBuilder;

#[test]
fn item_to_file_finder_find_item_if_present() {
	TestBuilder::default().with_file_ast().execute(|builder| {
		let item: Item = parse_quote! { trait A { fn some_func(&self); } };

		let item_to_file: ItemToFile = item.into();

		let ast = builder.get_ref_ast_file("file.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&item_to_file);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_file_finder_find_item_if_present_despite_docs() {
	TestBuilder::default().with_file_ast().execute(|builder| {
		let item: Item = parse_quote! { use std::fs; };

		let item_to_file: ItemToFile = item.into();

		let ast = builder.get_ref_ast_file("file.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&item_to_file);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_file_finder_cannot_find_item_if_not_present() {
	TestBuilder::default().with_file_ast().execute(|builder| {
		let item: Item = parse_quote! { fn non_existing_function() -> i32 { 0 } };

		let item_to_file: ItemToFile = item.into();

		let ast = builder.get_ref_ast_file("file.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&item_to_file);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_file_mutate_works_at_end() {
	TestBuilder::default().with_file_ast().execute(|mut builder| {
		let new_item: Item = parse_quote! {
			fn new_function() -> i32 { 42 }
		};
		let item_to_file: ItemToFile = new_item.clone().into();

		let ast = builder.get_mut_ast_file("file.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_file);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_file);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_file);
		assert!(finder.find(ast));

		let last_item = ast.items.last().expect("File must have at least one item; qed;");
		assert_eq!(*last_item, new_item);
	});
}
