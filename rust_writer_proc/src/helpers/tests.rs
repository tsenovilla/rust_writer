// SPDX-License-Identifier: GPL-3.0

use super::*;

#[test]
fn remove_impl_from_attr_works() {
	let struct_without_impl_from: ItemStruct = parse_quote!(
		#[some_attr]
		struct MyStruct {
			a: u8,
			b: u16,
		}
	);

	let mut struct_with_impl_from: ItemStruct = parse_quote!(
		#[some_attr]
		#[impl_from]
		struct MyStruct {
			a: u8,
			b: u16,
		}
	);

	remove_impl_from_attr(&mut struct_with_impl_from);
	assert_eq!(struct_without_impl_from, struct_with_impl_from);

	// The second time it's called, it doesn't have any effect as #[impl_from] isn't there anymore.
	remove_impl_from_attr(&mut struct_with_impl_from);
	assert_eq!(struct_without_impl_from, struct_with_impl_from);
}

#[test]
fn resolve_implementors_for_struct_empty_iterator() {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo {}
	};

	let resolved = resolve_implementors_for_struct(vec![].into_iter(), &item_struct);

	assert!(resolved.implementors_idents.is_empty());
	assert!(resolved.implementors_types_paths.is_empty());
	assert!(resolved.implementors_introduced_generics.is_empty());
}

#[test]
fn resolve_implementors_for_struct_duplicate_implementors() {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo;
	};

	let path1: Path = parse_quote! { Bar };
	let path2: Path = parse_quote! { Bar };
	let path3: Path = parse_quote! { Bar };

	let resolved =
		resolve_implementors_for_struct(vec![&path1, &path2, &path3].into_iter(), &item_struct);

	assert_eq!(resolved.implementors_idents[0].to_string(), "bar");
	assert_eq!(resolved.implementors_idents[1].to_string(), "bar_1");
	assert_eq!(resolved.implementors_idents[2].to_string(), "bar_2");
}

#[test]
fn resolve_implementors_for_struct_empty_generics() {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo {}
	};

	let path: Path = parse_quote! { Quux<> };

	let resolved = resolve_implementors_for_struct(vec![&path].into_iter(), &item_struct);

	assert_eq!(resolved.implementors_idents[0].to_string(), "quux");

	assert_eq!(resolved.implementors_types_paths[0], path);

	assert!(resolved.implementors_introduced_generics.is_empty());
}

#[test]
fn resolve_implementors_for_struct_no_new_generics() {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo<'a, 'b, T> {}
	};

	let path1: Path = parse_quote! { Baz<T> };
	let path2: Path = parse_quote! { Bar<'a, 'b>};

	let resolved = resolve_implementors_for_struct(vec![&path1, &path2].into_iter(), &item_struct);

	assert!(resolved.implementors_introduced_generics.is_empty());
}

#[test]
fn resolve_implementors_for_struct_new_generics_introduced_just_once() {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo {}
	};

	let path1: Path = parse_quote! { Baz<'a, T> };
	let path2: Path = parse_quote! { Bar<'a, 'b, T, U>};

	let resolved = resolve_implementors_for_struct(vec![&path1, &path2].into_iter(), &item_struct);

	let expected_implementors_introduced_generics: Vec<GenericParam> =
		vec![parse_quote!('a), parse_quote!(T), parse_quote!('b), parse_quote!(U)];

	assert_eq!(
		resolved.implementors_introduced_generics,
		expected_implementors_introduced_generics
	);
}

#[test]
fn resolve_implementors_for_struct_lifetimes_are_added_to_implementor_path_generics_at_the_beginning(
) {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo {}
	};

	let path1: Path = parse_quote! { Alpha<'a, 'b> };

	let resolved = resolve_implementors_for_struct(vec![&path1].into_iter(), &item_struct);

	let expected_implementors_types_paths: Vec<Path> = vec![parse_quote!(Alpha<'b, 'a>)];

	assert_eq!(resolved.implementors_types_paths, expected_implementors_types_paths);
}

#[test]
fn resolve_implementors_for_struct_generics_types_are_added_to_new_generic_while_path_keeps_just_ident(
) {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo {}
	};

	let path1: Path = parse_quote! { Beta<U: std::fmt::Debug, V: Clone> };

	let resolved = resolve_implementors_for_struct(vec![&path1].into_iter(), &item_struct);

	let expected_implementors_types_paths: Vec<Path> = vec![parse_quote!(Beta<U, V>)];

	let expected_implementors_introduced_generics: Vec<GenericParam> =
		vec![parse_quote!(U: std::fmt::Debug), parse_quote!(V: Clone)];

	assert_eq!(resolved.implementors_types_paths, expected_implementors_types_paths);
	assert_eq!(
		resolved.implementors_introduced_generics,
		expected_implementors_introduced_generics
	);
}

#[test]
fn resolve_implementors_for_struct_complete_test() {
	let item_struct: ItemStruct = parse_quote! {
		struct Foo<'a,T> {}
	};

	let path1: Path = parse_quote! { Alpha<'a, 'b> };
	let path2: Path = parse_quote! { Beta<'b, U: std::fmt::Debug> };

	let resolved = resolve_implementors_for_struct(vec![&path1, &path2].into_iter(), &item_struct);

	assert_eq!(resolved.implementors_idents[0].to_string(), "alpha");
	assert_eq!(resolved.implementors_idents[1].to_string(), "beta");

	let expected_implementors_types_paths: Vec<Path> =
		vec![parse_quote!(Alpha<'b, 'a>), parse_quote!(Beta<'b, U>)];

	let expected_implementors_introduced_generics: Vec<GenericParam> =
		vec![parse_quote!('b), parse_quote!(U: std::fmt::Debug)];

	assert_eq!(resolved.implementors_types_paths, expected_implementors_types_paths);
	assert_eq!(
		resolved.implementors_introduced_generics,
		expected_implementors_introduced_generics
	);
}

#[test]
fn add_new_implementors_generics_complete_test() {
	let mut item_struct: ItemStruct = parse_quote! {
		struct Foo<'a,T, U> {}
	};

	let new_implementor_generics: Vec<GenericParam> =
		vec![parse_quote!(S), parse_quote!('b), parse_quote!('c), parse_quote!(V), parse_quote!(W)];

	let expected_item_struct: ItemStruct = parse_quote! {
		struct Foo<'c,'b,'a,T, U, S, V, W> {}
	};

	add_new_implementors_generics(&mut item_struct, new_implementor_generics);

	assert_eq!(item_struct, expected_item_struct);
}
