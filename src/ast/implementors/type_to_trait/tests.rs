// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::{
	ast::{
		finder::{Finder, ToFind},
		mutator::{Mutator, ToMutate},
	},
	test_builder::TestBuilder,
};
use syn::{parse_quote, TraitItem};

#[test]
fn type_to_trait_mutate_works() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let type_to_trait: TypeToTrait =
			("MyTrait".to_owned(), TraitItem::Type(parse_quote! {type Something: From<String>;}))
				.into();

		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&type_to_trait);
		assert!(!finder.find(ast));

		let mutator = Mutator::default().to_mutate(type_to_trait.clone());
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&type_to_trait);
		assert!(finder.find(ast));
	});
}
