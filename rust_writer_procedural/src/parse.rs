// SPDX-License-Identifier: GPL-3.0

use crate::helpers::{self, ResolvedImplementors};
use syn::{
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::Punctuated,
	Error, Field, Fields, FieldsNamed, GenericParam, Ident, Index, ItemStruct, Path, Result, Token,
	Type, TypeArray, WhereClause,
};

// A single attribute in the #[mutator]/#[finder] macros
pub enum MacroAttr {
	CrateImplementor(Path),
	LocalImplementor(Path),
}

impl Parse for MacroAttr {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek(Ident) && input.peek2(Token![=]) {
			let ident: Ident = input.parse().expect("The lookahead guarantees this is Ok; qed;");
			let _eq: Token![=] = input.parse().expect("The lookeahead guarantees this is Ok; qed;");
			let path: Path = input.parse()?;
			if ident == "local" {
				Ok(MacroAttr::LocalImplementor(path))
			} else {
				Err(Error::new(ident.span(), "Expected 'local' as key."))
			}
		} else {
			let path: Path = input.parse()?;
			Ok(MacroAttr::CrateImplementor(path))
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
			.contains(&parse_quote!(#[rust_writer_procedural::already_expanded])) ||
			item_struct.attrs.contains(&parse_quote!(#[already_expanded]));

		let already_impl_from = item_struct
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

// The content of #[finder]/#[mutator] macros parsed
pub(crate) struct MacroParsed {
	pub(crate) crate_implementors_idents: Vec<Ident>,
	pub(crate) local_implementors_idents: Vec<Ident>,
	pub(crate) struct_: ItemStruct,
	pub(crate) already_expanded: bool,
	pub(crate) impl_from: bool,
	pub(crate) one: Index,
	pub(crate) implementors_count: Index,
	pub(crate) crate_implementors_indexes: Vec<Index>,
	pub(crate) local_implementors_indexes: Vec<Index>,
	pub(crate) generics_declarations: Punctuated<GenericParam, Token![,]>,
	pub(crate) generics_idents: Punctuated<GenericParam, Token![,]>,
	pub(crate) where_clause: WhereClause,
	pub(crate) new_struct_fields: Punctuated<Field, Token![,]>,
}

// The content of #[impl_finder]/#[impl_mutator] macros
pub(crate) struct MacroImplParsed {
	pub(crate) struct_: ItemStruct,
	pub(crate) generics_declarations: Punctuated<GenericParam, Token![,]>,
	pub(crate) generics_idents: Punctuated<GenericParam, Token![,]>,
	pub(crate) where_clause: WhereClause,
}

impl MacroParsed {
	pub(crate) fn try_from(attrs: MacroAttrs, mut struct_: ItemStruct) -> Result<Self> {
		let (already_expanded, impl_from) = attrs.validate_struct(&mut struct_)?.parse();

		let ResolvedImplementors {
			new_implementors_idents: new_crate_implementors_idents,
			implementors_idents: crate_implementors_idents,
			new_implementors: new_crate_implementors,
		} = helpers::resolve_implementors_for_struct(
			attrs.0.iter().filter_map(|macro_attr| match macro_attr {
				MacroAttr::CrateImplementor(path) => Some(path),
				_ => None,
			}),
			&mut struct_,
		);

		let ResolvedImplementors {
			new_implementors_idents: new_local_implementors_idents,
			implementors_idents: local_implementors_idents,
			new_implementors: new_local_implementors,
		} = helpers::resolve_implementors_for_struct(
			attrs.0.iter().filter_map(|macro_attr| match macro_attr {
				MacroAttr::LocalImplementor(path) => Some(path),
				_ => None,
			}),
			&mut struct_,
		);

		let one = Index::from(1);
		let struct_vis = &struct_.vis;

		let implementors_count =
			Index::from(crate_implementors_idents.len() + local_implementors_idents.len());

		let crate_implementors_indexes: Vec<Index> =
			(0..crate_implementors_idents.len()).map(Index::from).collect();

		let local_implementors_indexes: Vec<Index> = (crate_implementors_idents.len()..
			crate_implementors_idents.len() + local_implementors_idents.len())
			.map(Index::from)
			.collect();

		let (generics_declarations, generics_idents, where_clause) =
			rustilities::parsing::extract_generics(&struct_.generics);

		let where_clause = where_clause.unwrap_or(parse_quote! {where});

		let mut new_struct_fields: Punctuated<Field, Token![,]> = new_crate_implementors_idents
			.iter()
			.zip(new_crate_implementors)
			.map::<Field, _>(|(name, implementor)| parse_quote!(#struct_vis #name: #implementor))
			.collect();

		new_local_implementors_idents.iter().zip(new_local_implementors).for_each(
			|(name, implementor)| {
				new_struct_fields.push(parse_quote!(#struct_vis #name: #implementor))
			},
		);

		Ok(Self {
			crate_implementors_idents,
			local_implementors_idents,
			struct_,
			already_expanded,
			impl_from,
			one,
			implementors_count,
			crate_implementors_indexes,
			local_implementors_indexes,
			generics_declarations,
			generics_idents,
			where_clause,
			new_struct_fields,
		})
	}
}

impl MacroImplParsed {
	pub(crate) fn try_from(struct_: ItemStruct, field_ident: &str) -> Result<Self> {
		match &struct_.fields {
			Fields::Named(FieldsNamed { named, .. })
				if named.iter().any(|field| {
					field.ident.as_ref().expect("Named fields have ident; qed;") == field_ident &&
						matches!(&field.ty, Type::Array(TypeArray { elem, .. })
							if matches!(&**elem, Type::Path(path) if path.path.is_ident("bool"))
						)
				}) =>
			{
				let (generics_declarations, generics_idents, where_clause) =
					rustilities::parsing::extract_generics(&struct_.generics);

				let where_clause = where_clause.unwrap_or(parse_quote! {where});

				Ok(Self { struct_, generics_idents, where_clause, generics_declarations })
			},
			_ => Err(Error::new(
				struct_.ident.span(),
				format!("Expected a field named {} being [bool;N] inside struct.", field_ident),
			)),
		}
	}
}
