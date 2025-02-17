// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::test_builder::{TestBuilder, Visitor, WithItems};
use syn::{parse_quote, visit::Visit, TraitItem};

#[test]
fn type_to_trait_works() {
	TestBuilder::default().with_trait_ast().execute(|mut builder| {
		let type_ = TraitItem::Type(parse_quote! {type Something: From<String>;});
		let ast = builder.get_mut_ast_file("trait.rs").expect("This exists; qed;");
		let mut visitor = Visitor::default();
		visitor.with_items(&type_);
		visitor.visit_file(ast);
		assert!(!visitor.found());

		let type_to_trait: TypeToTrait = ("MyTrait".to_owned(), type_.clone()).into();
    let expander = Expander::default().to_modify(type_to_trait);
    expander.expand(ast);

		let mut visitor = Visitor::default();
		visitor.with_items(&type_);
		visitor.visit_file(ast);
		assert!(visitor.found());
	});
}
