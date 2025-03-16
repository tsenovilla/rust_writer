// SPDX-License-Identifier: GPL-3.0

use syn::{
	parse_quote, punctuated::Punctuated, Error, Fields, FieldsNamed, GenericParam, ItemStruct,
	Result, Token, Type, TypeArray, WhereClause,
};

// The content of #[impl_finder]/#[impl_mutator] macros
pub(crate) struct MacroImplParsed {
	pub(crate) struct_: ItemStruct,
	pub(crate) generics_declarations: Punctuated<GenericParam, Token![,]>,
	pub(crate) generics_idents: Punctuated<GenericParam, Token![,]>,
	pub(crate) where_clause: WhereClause,
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
