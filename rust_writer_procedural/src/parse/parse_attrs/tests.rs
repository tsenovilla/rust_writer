// SPDX-License-Identifier: GPL-3.0

use super::*;

fn create_macro_attrs(paths: Vec<Path>, local: Vec<bool>) -> MacroAttrs {
	let mut punct = syn::punctuated::Punctuated::new();
	for (path, is_local) in paths.into_iter().zip(local.into_iter()) {
		let macro_attr = if is_local {
			MacroAttr::LocalImplementor(path)
		} else {
			MacroAttr::CrateImplementor(path)
		};
		punct.push(macro_attr);
	}
	MacroAttrs(punct)
}

#[test]
fn inner_attr_parse_complete_test() {
	let inner_attr = InnerAttr::Nothing;

	assert_eq!(inner_attr.parse(), (false, false));

	let inner_attr = InnerAttr::AlreadyExpanded;

	assert_eq!(inner_attr.parse(), (true, false));

	let inner_attr = InnerAttr::ImplFrom;

	assert_eq!(inner_attr.parse(), (false, true));
}

#[test]
fn validate_struct_unit_struct_without_flags() {
	let item_struct: ItemStruct = parse_quote! {
		struct UnitStruct;
	};

	let paths: Vec<Path> = vec![parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, true]);

	assert_eq!(
		macro_attrs.validate_struct(&item_struct).expect("This is Ok; qed;"),
		InnerAttr::Nothing
	);
}

#[test]
fn validate_struct_unit_struct_with_impl_from() {
	let item_struct: ItemStruct = parse_quote! {
		#[impl_from]
		struct UnitStruct;
	};

	let paths: Vec<Path> = vec![parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, false]);

	assert_eq!(
		macro_attrs.validate_struct(&item_struct).expect("This is Ok; qed;"),
		InnerAttr::ImplFrom
	);
}

#[test]
fn validate_struct_unit_struct_with_already_expanded_errors() {
	let item_struct: ItemStruct = parse_quote! {
		#[already_expanded]
		struct UnitStruct;
	};

	let paths: Vec<Path> = vec![parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, false]);

	let res = macro_attrs.validate_struct(&item_struct);
	assert!(
		matches!(res, Err(err) if err.to_string() == "Cannot use #[already_expanded] attribute in an unit struct")
	);
}

#[test]
fn validate_struct_named_struct_already_expanded_valid() {
	let item_struct: ItemStruct = parse_quote! {
		#[already_expanded]
		struct NamedStruct {
	  other_field: [bool; 73],
			field1: A,
			field2: B,
		}
	};

	let paths: Vec<Path> = vec![parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, true]);

	assert_eq!(
		macro_attrs.validate_struct(&item_struct).expect("This is Ok; qed;"),
		InnerAttr::AlreadyExpanded
	);
}

#[test]
fn validate_struct_named_struct_already_expanded_mismatch() {
	let item_struct: ItemStruct = parse_quote! {
		#[already_expanded]
		struct NamedStruct {
			field1: B,
			field2: A,
		}
	};

	let paths: Vec<Path> = vec![parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, false]);

	let res = macro_attrs.validate_struct(&item_struct);
	assert!(matches!(res, Err(err) if err.to_string() == COMBINED_MACROS_MSG));
}

#[test]
fn validate_struct_named_struct_without_flags() {
	let item_struct: ItemStruct = parse_quote! {
		struct NamedStruct {
			field1: A,
			field2: B,
		}
	};

	let paths: Vec<Path> = vec![parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, false]);

	assert_eq!(
		macro_attrs.validate_struct(&item_struct).expect("This is Ok; qed;"),
		InnerAttr::Nothing
	);
}

#[test]
fn validate_struct_named_struct_with_impl_from() {
	let item_struct: ItemStruct = parse_quote! {
		#[impl_from]
		struct NamedStruct {
			field1: A,
			field2: B,
		}
	};

	let paths: Vec<Path> = vec![parse_quote!(C), parse_quote!(A), parse_quote!(B)];
	let macro_attrs = create_macro_attrs(paths, vec![false, false]);

	assert_eq!(
		macro_attrs.validate_struct(&item_struct).expect("This is Ok; qed;"),
		InnerAttr::ImplFrom
	);
}

#[test]
fn validate_struct_tuple_struct_error() {
	let item_struct: ItemStruct = parse_quote! {
		struct Struct(u8);
	};

	let paths: Vec<Path> = vec![];
	let macro_attrs = create_macro_attrs(paths, vec![false, false]);

	let res = macro_attrs.validate_struct(&item_struct);
	assert!(
		matches!(res, Err(err) if err.to_string() == "Expected unit struct or named-field struct.")
	);
}
