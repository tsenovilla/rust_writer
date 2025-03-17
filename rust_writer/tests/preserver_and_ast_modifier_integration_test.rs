// SPDX-License-Identifier: GPL-3.0

use quote::quote;
use rust_writer::{
	ast::{
		implementors::{ItemToFile, ItemToImpl, TokenStreamToMacro},
		macros::mutator,
		mutator::{Mutator, ToMutate},
	},
	preserver::types::Preserver,
};
use syn::{parse_quote, visit_mut::VisitMut, ImplItem, Item};
use test_builder::TestBuilder;

#[mutator(TokenStreamToMacro, ItemToFile, ItemToImpl<'a>)]
#[impl_from]
struct TestMutator;

#[test]
fn preserver_and_ast_modifier_integration() {
	TestBuilder::default()
		.with_complete_file()
		.with_expanded_file()
		.execute(|builder| {
			let complete_file_path =
				builder.tempfile_path("complete_file.rs").expect("This exists; qed;");

			let expanded_file_path =
				builder.tempfile_path("expanded_file.rs").expect("This exists; qed;");

			let expected_code =
				std::fs::read_to_string(&expanded_file_path).expect("File should be readable");

			let mut preserver1 = Preserver::new("impl MyTrait for MyStruct");
			preserver1.add_inners(vec!["fn trait_method"]);
			let preserver2 = Preserver::new("fn main");

			let mut ast = rust_writer::preserver::preserve_and_parse(
				complete_file_path,
				vec![preserver1, preserver2],
			)
			.expect("Preserves should be applied; qed;");

			let item_to_impl: ItemToImpl = (
				Some("MyTrait"),
				"MyStruct",
				ImplItem::Fn(parse_quote! {
				fn func(&self) -> bool{
							false
						 }
				  }),
			)
				.into();

			let token_stream_to_macro: TokenStreamToMacro = (
				parse_quote!(my_macro),
				None,
				quote! {
					struct SomeStruct {
						field: u8,
						string: String
					}
				},
			)
				.into();

			let item: Item = parse_quote!(
				use std::path::Path;
			);
			let item_to_file: ItemToFile = item.into();

			let test_mutator: TestMutator =
				(token_stream_to_macro, item_to_file, item_to_impl).into();

			let mut mutator: TestMutatorMutatorWrapper =
				Mutator::default().to_mutate(&test_mutator).into();
			assert!(mutator.mutate(&mut ast).is_ok());

			assert!(rust_writer::preserver::resolve_preserved(&ast, complete_file_path).is_ok());

			// The preserved resolver cannot ensure formatting at 100%, but if we format it we can
			// assert equality with the resolved file
			assert!(rustilities::fmt::format_dir(builder.tempdir_path()).is_ok());

			let actual_code =
				std::fs::read_to_string(complete_file_path).expect("File should be readable");

			assert_eq!(actual_code, expected_code);
		});
}
