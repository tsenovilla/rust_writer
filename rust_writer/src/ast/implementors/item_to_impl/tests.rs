// SPDX-License-Identifier: GPL-3.0

use super::*;
use crate::Error;
use syn::{parse_quote, Item};
use test_builder::TestBuilder;

#[test]
fn path_segment_finder_finds_with_trait() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let mut finder = PathSegmentFinder {
			found: [false; 2],
			trait_name: Some("SomeTrait"),
			implementor_name: "SomeImplementor",
		};

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed");

		// ast.items[0] is impl SomeTrait for SomeImplementor{..}
		match &ast.items[0] {
			Item::Impl(ref item_impl) => finder.find_impl_paths(item_impl),
			_ => unreachable!("By construction this is an impl block; qed;"),
		}

		assert!(finder.found.iter().all(|&x| x));
	});
}

#[test]
fn path_segment_finder_finds_without_trait() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let mut finder = PathSegmentFinder {
			found: [false; 2],
			trait_name: None,
			implementor_name: "SomeImplementor",
		};

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed");

		// ast.items[1] is impl SomeImplementor{..}
		match &ast.items[1] {
			Item::Impl(ref item_impl) => finder.find_impl_paths(item_impl),
			_ => unreachable!("By construction this is an impl block; qed;"),
		}

		assert!(finder.found.iter().all(|&x| x));
	});
}

#[test]
fn path_segment_finder_with_trait_doesnt_find_if_not_trait() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let mut finder = PathSegmentFinder {
			found: [false; 2],
			trait_name: Some("SomeTrait"),
			implementor_name: "SomeImplementor",
		};

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed");

		match &ast.items[1] {
			Item::Impl(ref item_impl) => finder.find_impl_paths(item_impl),
			_ => unreachable!("By construction this is an impl block; qed;"),
		}

		assert!(!finder.found[0]);
	});
}

#[test]
fn path_segment_finder_without_trait_doesnt_find_if_trait() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let mut finder = PathSegmentFinder {
			found: [false; 2],
			trait_name: None,
			implementor_name: "SomeImplementor",
		};

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed");

		match &ast.items[0] {
			Item::Impl(ref item_impl) => finder.find_impl_paths(item_impl),
			_ => unreachable!("By construction this is an impl block; qed;"),
		}

		assert!(!finder.found[0]);
	});
}

#[test]
fn path_segment_finder_doesnt_find_bad_implementor_name() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let mut finder = PathSegmentFinder {
			found: [false; 2],
			trait_name: None,
			implementor_name: "SomeImplemen",
		};

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed");

		match &ast.items[1] {
			Item::Impl(ref item_impl) => finder.find_impl_paths(item_impl),
			_ => unreachable!("By construction this is an impl block; qed;"),
		}

		assert!(finder.found[0]);
		assert!(!finder.found[1]);
	});
}

#[test]
fn item_to_impl_with_trait_finder_find_item_if_present() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_with_trait: ItemToImpl = (
			Some("SomeTrait"),
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_impl_with_trait_finder_cannot_find_item_if_trait_name_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_with_trait: ItemToImpl = (
			Some("SomTrait"),
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_with_trait_finder_cannot_find_item_if_implementor_name_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_with_trait: ItemToImpl = (
			Some("SomeTrait"),
			"SomImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_with_trait_finder_cannot_find_item_if_item_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_with_trait: ItemToImpl = (
			Some("SomeTrait"),
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn som_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_with_trait_mutate_works() {
	TestBuilder::default().with_impl_block_ast().execute(|mut builder| {
		let item_to_impl_with_trait: ItemToImpl = (
			Some("SomeTrait"),
			"SomeImplementor",
			ImplItem::Type(parse_quote! {type Something = String;}),
		)
			.into();

		let ast = builder.get_mut_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_impl_with_trait);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_impl_with_trait_mutate_fails_if_cannot_find_impl_block() {
	TestBuilder::default().with_impl_block_ast().execute(|mut builder| {
		let item_to_impl_with_trait: ItemToImpl = (
			Some("SomTrait"),
			"SomeImplementor",
			ImplItem::Type(parse_quote! {type Something = String;}),
		)
			.into();

		let ast = builder.get_mut_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_impl_with_trait);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
			if msg == format!("Cannot mutate using Mutator: {:?}", item_to_impl_with_trait)
		));

		let mut finder = Finder::default().to_find(&item_to_impl_with_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_without_trait_finder_find_item_if_present() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_without_trait: ItemToImpl = (
			None,
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_super_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_impl_without_trait_finder_cannot_find_item_if_implementor_name_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_without_trait: ItemToImpl = (
			None,
			"SomImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_super_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_without_trait_finder_cannot_find_item_if_item_incorrect() {
	TestBuilder::default().with_impl_block_ast().execute(|builder| {
		let item_to_impl_without_trait: ItemToImpl = (
			None,
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
			.into();

		let ast = builder.get_ref_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(!finder.find(ast));
	});
}

#[test]
fn item_to_impl_without_trait_mutate_works() {
	TestBuilder::default().with_impl_block_ast().execute(|mut builder| {
		let item_to_impl_without_trait: ItemToImpl =
			(None, "SomeImplementor", ImplItem::Type(parse_quote! {type Something = String;}))
				.into();

		let ast = builder.get_mut_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_impl_without_trait);
		assert!(mutator.mutate(ast).is_ok());

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(finder.find(ast));
	});
}

#[test]
fn item_to_impl_without_trait_mutate_fails_if_cannot_find_impl_block() {
	TestBuilder::default().with_impl_block_ast().execute(|mut builder| {
		let item_to_impl_without_trait: ItemToImpl =
			(None, "SoeImplementor", ImplItem::Type(parse_quote! {type Something = String;}))
				.into();

		let ast = builder.get_mut_ast_file("impl_block.rs").expect("This exists; qed;");

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(!finder.find(ast));

		let mut mutator = Mutator::default().to_mutate(&item_to_impl_without_trait);
		assert!(matches!(
			mutator.mutate(ast),
			Err(Error::Descriptive(msg))
			if msg == format!("Cannot mutate using Mutator: {:?}", item_to_impl_without_trait)
		));

		let mut finder = Finder::default().to_find(&item_to_impl_without_trait);
		assert!(!finder.find(ast));
	});
}
