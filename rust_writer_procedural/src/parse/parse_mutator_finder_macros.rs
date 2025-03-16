// SPDX-License-Identifier: GPL-3.0

use crate::{
	helpers::{self, ResolvedImplementors},
	parse::{MacroAttr, MacroAttrs},
};
use syn::{
	parse_quote, punctuated::Punctuated, Field, GenericParam, Ident, Index, ItemStruct, Result,
	Token, WhereClause,
};

// The content of #[finder]/#[mutator] macros parsed
pub(crate) struct MacroFinderMutatorParsed {
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

impl MacroFinderMutatorParsed {
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
