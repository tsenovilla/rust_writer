// SPDX-License-Identifier: GPL-3.0

use syn::{
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::Punctuated,
	Error, Fields, FieldsNamed, Ident, ItemStruct, Path, Result, Token, Type,
};

pub(crate) enum MacroAttr {
	CrateImplementor(Path),
	LocalImplementor(Path),
}

impl Parse for MacroAttr {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek(Ident) && input.peek2(Token![=]) {
			let ident: Ident = input.parse()?;
			let _eq: Token![=] = input.parse()?;
			let path: Path = input.parse()?;
			if ident == "local" {
				Ok(MacroAttr::LocalImplementor(path))
			} else {
				Err(Error::new(ident.span(), "Expected 'local' as key"))
			}
		} else {
			let path: Path = input.parse()?;
			Ok(MacroAttr::CrateImplementor(path))
		}
	}
}

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
pub(crate) enum InnerAttr {
	Unit,
	NotUnit,
	ImplFrom,
	ImplFromNotUnit,
}

const IMPL_FROM_ERR_MSG: &str = "#[impl_from] is only allowed in structs annotated with #[mutator]/#[finder] at most once or unit structs annotated with #[mutator] and #[finder] using the same implementors.";

impl MacroAttrs {
	pub(crate) fn validate_struct(&self, item_struct: &mut ItemStruct) -> Result<InnerAttr> {
		let already_expanded = item_struct
			.attrs
			.contains(&parse_quote!(#[rust_writer_procedural::already_expanded])) ||
			item_struct.attrs.contains(&parse_quote!(#[already_expanded]));
		let impl_from = item_struct.attrs.contains(&parse_quote!(#[impl_from]));
		match (&item_struct.fields, already_expanded, impl_from) {
			(Fields::Unit, false, false) => Ok(InnerAttr::Unit),
			(Fields::Unit, false, true) => {
				item_struct_remove_impl_from_attr(item_struct);
				Ok(InnerAttr::ImplFrom)
			},
			(Fields::Unit, true, _) => Err(Error::new(
				item_struct.ident.span(),
				"Cannot use #[already_expanded] attribute in an unit struct",
			)),
			(Fields::Named(FieldsNamed { named, .. }), true, true) => {
				let struct_values = named
					.iter()
					.map(|field| match &field.ty {
						Type::Path(path) => Ok(&path.path),
						_ => Err(Error::new(item_struct.ident.span(), IMPL_FROM_ERR_MSG)),
					})
					.collect::<Result<Vec<&Path>>>()?;

				let implementors_vec: Vec<&Path> = self
					.0
					.iter()
					.map(|macro_attr| match macro_attr {
						MacroAttr::CrateImplementor(path) => path,
						MacroAttr::LocalImplementor(path) => path,
					})
					.collect();

				if struct_values.len() != implementors_vec.len() ||
					struct_values.iter().any(|value| !implementors_vec.contains(value))
				{
					return Err(Error::new(item_struct.ident.span(), IMPL_FROM_ERR_MSG));
				}
				item_struct_remove_impl_from_attr(item_struct);
				Ok(InnerAttr::ImplFromNotUnit)
			},
			(Fields::Named(_), _, false) => Ok(InnerAttr::NotUnit),
			(Fields::Named(_), false, true) => {
				item_struct_remove_impl_from_attr(item_struct);
				Ok(InnerAttr::ImplFromNotUnit)
			},
			_ => Err(Error::new(
				item_struct.ident.span(),
				"Expected unit struct or named-field struct.",
			)),
		}
	}
}

fn item_struct_remove_impl_from_attr(struct_: &mut ItemStruct) {
	struct_.attrs = struct_
		.attrs
		.clone()
		.into_iter()
		.filter(|attr| !attr.path().is_ident("impl_from"))
		.collect()
}
