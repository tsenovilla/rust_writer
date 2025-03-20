// SPDX-License-Identifier: GPL-3.0

use rust_writer::{
	ast::{
		finder::{Finder, ToFind},
		implementors::ItemToTrait,
	},
	Error,
};
use rust_writer_procedural::local_mutator;
use syn::{parse_quote, visit_mut::VisitMut, ItemTrait, TraitItem};
use test_builder::TestBuilder;

// A custom mutator emulating ItemToTrait
#[local_mutator]
#[derive(Debug)]
struct SomeStruct<'a, T: Clone + std::fmt::Debug> {
	mutated: [bool; 1],
	trait_name: &'a str,
	item_trait: TraitItem,
	#[allow(dead_code)]
	just_extra_data: T,
}

impl<'a, T: Clone + std::fmt::Debug> VisitMut for SomeStruct<'a, T> {
	fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
		if item_trait.ident == self.trait_name {
			self.mutated[0] = true;
			item_trait.items.push(self.item_trait.clone());
		}
	}
}

#[test]
fn local_mutator_struct_mutates() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let mut some_struct = SomeStruct {
			mutated: [false],
			trait_name: "MyTrait",
			item_trait: TraitItem::Type(parse_quote! {type Something: From<String>;}),
			just_extra_data: 1u8,
		};

		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Something: From<String>;})).into();

		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));

		assert!(some_struct.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn local_mutator_struct_fails_if_unable_to_mutate() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let mut some_struct = SomeStruct {
			mutated: [false],
			trait_name: "Trait",
			item_trait: TraitItem::Type(parse_quote! {type Something: From<String>;}),
			just_extra_data: 1u8,
		};

		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Something: From<String>;})).into();

		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));

		let expected_msg = format!("Cannot mutate using Mutator: {:?}", some_struct);
		assert!(matches!(
			some_struct.mutate(ast),
			Err(Error::Descriptive(msg))
			if msg == expected_msg));

		let mut finder = Finder::default().to_find(&item_to_trait);
		assert!(!finder.find(ast));
	});
}
