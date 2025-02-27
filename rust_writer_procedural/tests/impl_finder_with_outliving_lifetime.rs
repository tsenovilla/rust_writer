// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::impl_finder;
use syn::{parse_quote, visit::Visit, ItemTrait, TraitItem};
use test_builder::TestBuilder;

// A custom finder emulating ItemToTrait
#[impl_finder('b:'a)]
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
fn impl_finder_struct_with_outliving_lifetime() {
	TestBuilder::default().with_trait_ast().execute(|builder| {
		let mut some_struct = SomeStruct {
			found: [false],
			trait_name: "MyTrait",
			item_trait: TraitItem::Type(parse_quote! {type Type1: From<String>;}),
			just_extra_data: 1u8,
		};

		let ast = builder.get_ref_ast_file("trait.rs").expect("This exists; qed;");

		assert!(some_struct.find(ast));
	});
}
