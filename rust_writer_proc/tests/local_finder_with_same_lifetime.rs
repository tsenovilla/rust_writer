// SPDX-License-Identifier: GPL-3.0

use rust_writer_proc::local_finder;
use syn::{parse_quote, visit::Visit, ItemTrait, TraitItem};
use test_builder::TestBuilder;

// A custom finder emulating ItemToTrait
#[local_finder('a)]
#[derive(Debug)]
struct SomeStruct<'a, T: Clone + std::fmt::Debug> {
	found: [bool; 1],
	trait_name: &'a str,
	item_trait: TraitItem,
	#[allow(dead_code)]
	just_extra_data: T,
}

impl<'a, T: Clone + std::fmt::Debug> Visit<'a> for SomeStruct<'a, T> {
	fn visit_item_trait(&mut self, item_trait: &'a ItemTrait) {
		if item_trait.ident == self.trait_name && item_trait.items.contains(&self.item_trait) {
			self.found[0] = true;
		}
	}
}

#[test]
fn local_finder_struct_with_same_lifetime() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let mut some_struct = SomeStruct {
			found: [false],
			trait_name: "MyTrait",
			item_trait: TraitItem::Type(parse_quote! {type Type1: From<String>;}),
			just_extra_data: 1u8,
		};

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		assert!(some_struct.find(ast));

		// finder_reset method works
		assert_eq!(some_struct.found, [true]);
		some_struct.finder_reset();
		assert_eq!(some_struct.found, [false]);
	});
}
