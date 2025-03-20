// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::ast::implementors::ItemToTrait;
use syn::{parse_quote, TraitItem};
use test_builder::TestBuilder;

#[test]
fn mutator_works_for_a_implementor() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();

		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");

		let mut mutator = Mutator::default().to_mutate(&item_to_trait);
		assert!(mutator.mutate(ast).is_ok());

		assert!(mutator.mutated[0]);

		mutator.reset();

		assert!(!mutator.mutated[0]);
	});
}
