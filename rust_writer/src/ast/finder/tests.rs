// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::ast::implementors::ItemToTrait;
use syn::{parse_quote, TraitItem};
use test_builder::TestBuilder;

#[test]
fn finder_works_for_a_implementor() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(finder.find(ast));

		assert!(finder.found[0]);

		finder.reset();

		assert!(!finder.found[0]);
	});
}
