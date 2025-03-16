// SPDX-License-Identifier: GPL-3.0

use crate::helpers;
use syn::{
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::Punctuated,
	Error, Fields, FieldsNamed, ItemStruct, Path, Result, Token, Type,
};

mod keywords {
	syn::custom_keyword!(local);
}

// A single attribute in the #[mutator]/#[finder] macros
pub enum MacroAttr {
	CrateImplementor(Path),
	LocalImplementor(Path),
}

const MACRO_ATTR_PARSE_ERR: &str = "Expected a path to a rust_writer implementor or 'local =' followed by a local implementor path";

impl Parse for MacroAttr {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek(keywords::local) && input.peek2(Token![=]) {
			let _ident: keywords::local =
				input.parse().expect("The lookahead guarantees this is Ok; qed;");
			let _eq: Token![=] = input.parse().expect("The lookeahead guarantees this is Ok; qed;");
			let path: Path = input.parse()?;
			Ok(MacroAttr::LocalImplementor(path))
		} else {
			match input.parse::<Path>() {
				Ok(path) => Ok(MacroAttr::CrateImplementor(path)),
				Err(_) => Err(Error::new(input.span(), MACRO_ATTR_PARSE_ERR)),
			}
		}
	}
}

// The list of attributes in the #[mutator]/#[finder] macros
pub(crate) struct MacroAttrs(pub(crate) Punctuated<MacroAttr, Token![,]>);

impl Parse for MacroAttrs {
	fn parse(input: ParseStream) -> Result<Self> {
		let implementors = input.parse_terminated(MacroAttr::parse, Token![,])?;
		if implementors.len() < 2 {
			Err(Error::new(input.span(), "Expected at least two implementors."))
		} else {
			Ok(Self(implementors))
		}
	}
}

#[derive(PartialEq)]
pub enum InnerAttr {
	Nothing,
	AlreadyExpanded,
	ImplFrom,
}

type AlreadyExpandedStruct = bool;
type StructNeedsImplFrom = bool;

impl InnerAttr {
	pub fn parse(self) -> (AlreadyExpandedStruct, StructNeedsImplFrom) {
		match self {
			InnerAttr::Nothing => (false, false),
			InnerAttr::AlreadyExpanded => (true, false),
			InnerAttr::ImplFrom => (false, true),
		}
	}
}

const IMPL_FROM_ERR_MSG: &str = "#[impl_from] is only allowed in structs annotated with #[mutator]/#[finder] at most once or structs annotated with #[mutator] and #[finder] such that the outermost macro implementors set contains the innermost macro implementors set.";

impl MacroAttrs {
	pub(crate) fn validate_struct(&self, item_struct: &mut ItemStruct) -> Result<InnerAttr> {
		let already_expanded = item_struct
			.attrs
			.contains(&parse_quote!(#[rust_writer::ast::macros::already_expanded])) ||
			item_struct
				.attrs
				.contains(&parse_quote!(#[rust_writer_procedural::already_expanded])) ||
			item_struct.attrs.contains(&parse_quote!(#[already_expanded]));

		let already_impl_from = item_struct
			.attrs
			.contains(&parse_quote!(#[rust_writer::ast::macros::already_impl_from])) ||
			item_struct
				.attrs
				.contains(&parse_quote!(#[rust_writer_procedural::already_impl_from])) ||
			item_struct.attrs.contains(&parse_quote!(#[already_impl_from]));

		let impl_from = item_struct.attrs.contains(&parse_quote!(#[impl_from]));

		match (&item_struct.fields, already_expanded, already_impl_from, impl_from) {
			(_, _, true, true) => Err(Error::new(
				item_struct.ident.span(),
				"Cannot use #[impl_from] in an struct annotated with #[already_impl_from]",
			)),
			(Fields::Unit, true, _, _) => Err(Error::new(
				item_struct.ident.span(),
				"Cannot use #[already_expanded] attribute in an unit struct",
			)),
			(Fields::Unit, _, _, false) => Ok(InnerAttr::Nothing),
			(Fields::Unit, _, _, true) => {
				helpers::remove_impl_from_attr(item_struct);
				Ok(InnerAttr::ImplFrom)
			},
			(Fields::Named(FieldsNamed { named, .. }), true, true, _) => {
				// Just a toy path to include in struct_path_values instead of non_path arguments
				let toy_path: Path =
					parse_quote!(some::unlikely::used::path::segment::as_::implementor::name);
				// From block has been already implemented. Check that the implementors are all
				// contained in the struct, so the expansion won't break the From trait
				let struct_path_values: Vec<&Path> = named
					.iter()
					.map(|field| match &field.ty {
						// Not path types aren't interesting as previous macro won't introduce those
						// types from its implementors.
						Type::Path(path) => &path.path,
						_ => &toy_path,
					})
					.collect();

				let implementors_vec: Vec<&Path> = self
					.0
					.iter()
					.map(|macro_attr| match macro_attr {
						MacroAttr::CrateImplementor(path) => path,
						MacroAttr::LocalImplementor(path) => path,
					})
					.collect();

				if !implementors_vec.iter().all(|path| struct_path_values.contains(path)) {
					return Err(Error::new(item_struct.ident.span(), IMPL_FROM_ERR_MSG));
				}
				Ok(InnerAttr::AlreadyExpanded)
			},
			(Fields::Named(_), true, _, false) => Ok(InnerAttr::AlreadyExpanded),
			(Fields::Named(_), false, _, false) => Ok(InnerAttr::Nothing),
			(Fields::Named(_), _, _, true) => {
				helpers::remove_impl_from_attr(item_struct);
				Ok(InnerAttr::ImplFrom)
			},
			_ => Err(Error::new(
				item_struct.ident.span(),
				"Expected unit struct or named-field struct.",
			)),
		}
	}
}
