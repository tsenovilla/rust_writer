// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::Error;
use syn::parse_quote;
use test_builder::TestBuilder;

#[test]
fn token_stream_to_macro_finder_finds_token_stream_without_container() {
	TestBuilder::default().with_macro_ast().execute(|builder| {
		let token_stream_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, None, parse_quote! { type Type1 = From<String>; }).into();

		let ast = builder.get_ref_ast_file("macro.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&token_stream_to_macro);
		assert!(finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_finder_finds_token_stream_with_container() {
	TestBuilder::default().with_macro_ast().execute(|builder| {
		let token_stream_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, Some(parse_quote! { SomeEnum }), parse_quote! { A }).into();

		let ast = builder.get_ref_ast_file("macro.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&token_stream_to_macro);
		assert!(finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_finder_cannot_find_macro_if_macro_path_incorrect() {
	TestBuilder::default().with_macro_ast().execute(|builder| {
		let token_stream_to_macro: TokenStreamToMacro =
			(parse_quote! { other_macro }, None, parse_quote! { type Type1 = From<String>; })
				.into();

		let ast = builder.get_ref_ast_file("macro.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&token_stream_to_macro);
		assert!(!finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_finder_cannot_find_macro_if_token_stream_incorrect() {
	TestBuilder::default().with_macro_ast().execute(|builder| {
		let token_stream_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, None, parse_quote! { type Type1 = From<u8>; }).into();

		let ast = builder.get_ref_ast_file("macro.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&token_stream_to_macro);
		assert!(!finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_finder_cannot_find_inside_container_an_outer_stream() {
	TestBuilder::default().with_macro_ast().execute(|builder| {
		let token_stream_to_macro: TokenStreamToMacro = (
			parse_quote! { my_macro },
			Some(parse_quote! { SomeEnum }),
			parse_quote! { type Type1 = From<String> },
		)
			.into();

		let ast = builder.get_ref_ast_file("macro.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&token_stream_to_macro);
		assert!(!finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_finder_fails_if_searching_in_wrong_container() {
	TestBuilder::default().with_macro_ast().execute(|builder| {
		let token_stream_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, Some(parse_quote! { OtherEnum }), parse_quote! { A })
				.into();

		let ast = builder.get_ref_ast_file("macro.rs").expect("This exists; qed;");
		let mut finder = Finder::default().to_find(&token_stream_to_macro);
		assert!(!finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_mutate_works_without_container() {
	TestBuilder::default().with_macro_ast().execute(|mut builder| {
		let token_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, None, parse_quote! { D }).into();

		let ast = builder.get_mut_ast_file("macro.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&token_to_macro);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_mutate_works_with_container() {
	TestBuilder::default().with_macro_ast().execute(|mut builder| {
		let token_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, Some(parse_quote! { SomeEnum }), parse_quote! { D }).into();

		let ast = builder.get_mut_ast_file("macro.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&token_to_macro);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_mutate_fails_if_cannot_find_container() {
	TestBuilder::default().with_macro_ast().execute(|mut builder| {
		let token_to_macro: TokenStreamToMacro =
			(parse_quote! { my_macro }, Some(parse_quote! { UnexistingEnum }), parse_quote! { D })
				.into();

		let ast = builder.get_mut_ast_file("macro.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&token_to_macro);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
				if msg == format!("Cannot mutate using Mutator: {:?}", token_to_macro)
		));

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(!finder.find(ast));
	});
}

#[test]
fn token_stream_to_macro_mutate_fails_if_macro_not_found() {
	TestBuilder::default().with_macro_ast().execute(|mut builder| {
		let token_to_macro: TokenStreamToMacro = (
			parse_quote! { unexisting_macro },
			Some(parse_quote! { SomeEnum }),
			parse_quote! { D },
		)
			.into();

		let ast = builder.get_mut_ast_file("macro.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&token_to_macro);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
				if msg == format!("Cannot mutate using Mutator: {:?}", token_to_macro)
		));

		let mut finder = Finder::default().to_find(&token_to_macro);
		assert!(!finder.find(ast));
	});
}
