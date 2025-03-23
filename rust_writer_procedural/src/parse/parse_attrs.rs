// SPDX-License-Identifier: GPL-3.0

mod keywords {
	syn::custom_keyword!(local);
}

#[cfg(test)]
mod tests;

use syn::{
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::Punctuated,
	Error, Fields, FieldsNamed, GenericArgument, ItemStruct, Path, PathArguments, Result, Token,
	Type, TypePath,
};

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

#[derive(Debug, PartialEq)]
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

const COMBINED_MACROS_MSG: &str = "#[mutator]/#[finder] combination is only possible if the set of implementors in both attributes is the same.";

impl MacroAttrs {
	pub(crate) fn validate_struct(&self, item_struct: &ItemStruct) -> Result<InnerAttr> {
		let already_expanded =
			item_struct.attrs.contains(&parse_quote!(#[rust_writer::ast::already_expanded])) ||
				item_struct
					.attrs
					.contains(&parse_quote!(#[rust_writer_procedural::already_expanded])) ||
				item_struct.attrs.contains(&parse_quote!(#[already_expanded]));

		let impl_from = item_struct.attrs.contains(&parse_quote!(#[impl_from]));

		match (&item_struct.fields, already_expanded, impl_from) {
			(Fields::Unit, true, _) => Err(Error::new(
				item_struct.ident.span(),
				"Cannot use #[already_expanded] attribute in an unit struct",
			)),
			(Fields::Unit, _, false) => Ok(InnerAttr::Nothing),
			(Fields::Unit, _, true) => Ok(InnerAttr::ImplFrom),
			(Fields::Named(FieldsNamed { named, .. }), true, _) => {
				// Just a toy path to include in struct_path_values instead of non_path arguments
				let toy_path: Path =
					parse_quote!(some::unlikely::used::path::segment::as_::implementor::name);

				// The struct has been already expanded. #[mutator] and #[finder] can only be used
				// with the same implementors set can only be used with the same implementors
				// set..
				let struct_path_values: Vec<&Path> = named
					.iter()
					.map(|field| match &field.ty {
						// Not path types aren't interesting as previous macro won't introduce those
						// types from its implementors.
						Type::Path(path) => &path.path,
						_ => &toy_path,
					})
					.collect();

				let implementors_vec: Vec<Path> = self
					.0
					.iter()
					.map(|macro_attr| match macro_attr {
						MacroAttr::CrateImplementor(path) => path,
						MacroAttr::LocalImplementor(path) => path,
					})
					.map(|path| {
						// The struct path values only contains the generics idents, not the trait
						// bounds
						let mut path = path.clone();
						let last_path_segment = path
							.segments
							.last_mut()
							.expect("At this point, implementors are valid paths; qed;");

						if let PathArguments::AngleBracketed(ref mut generics) =
							last_path_segment.arguments
						{
							generics.args.iter_mut().for_each(|argument| {
								if let GenericArgument::Constraint(ref mut generic) = argument {
									let generic_ident = &generic.ident;
									let generic_as_path: TypePath = parse_quote! { #generic_ident };
									*argument = GenericArgument::Type(Type::Path(generic_as_path));
								}
							});
						}

						path
					})
					.collect();

				match struct_path_values.iter().position(|&path| *path == implementors_vec[0]) {
					Some(position)
						if struct_path_values[position..] ==
							implementors_vec.iter().collect::<Vec<&Path>>() =>
						Ok(InnerAttr::AlreadyExpanded),
					_ => Err(Error::new(item_struct.ident.span(), COMBINED_MACROS_MSG)),
				}
			},
			(Fields::Named(_), _, false) => Ok(InnerAttr::Nothing),
			(Fields::Named(_), _, true) => Ok(InnerAttr::ImplFrom),
			_ => Err(Error::new(
				item_struct.ident.span(),
				"Expected unit struct or named-field struct.",
			)),
		}
	}
}
