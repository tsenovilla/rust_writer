// SPDX-License-Identifier: GPL-3.0

use crate::{
	helpers::{self, ResolvedImplementors},
	parse::{MacroAttr, MacroAttrs},
};
use syn::{
	parse_quote, punctuated::Punctuated, Field, GenericParam, Ident, Index, ItemStruct, Result,
	Token,
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
	pub(crate) implementors_introduced_generics: Vec<GenericParam>,
	pub(crate) new_struct_fields: Punctuated<Field, Token![,]>,
}

impl MacroFinderMutatorParsed {
	pub(crate) fn try_from(attrs: MacroAttrs, struct_: ItemStruct) -> Result<Self> {
		let (already_expanded, impl_from) = attrs.validate_struct(&struct_)?.parse();

		let ResolvedImplementors {
			implementors_idents: crate_implementors_idents,
			implementors_types_paths: crate_implementors_types_paths,
			implementors_introduced_generics: crate_implementors_introduced_generics,
		} = helpers::resolve_implementors_for_struct(
			attrs.0.iter().filter_map(|macro_attr| match macro_attr {
				MacroAttr::CrateImplementor(path) => Some(path),
				_ => None,
			}),
			&struct_,
		);

		let ResolvedImplementors {
			implementors_idents: local_implementors_idents,
			implementors_types_paths: local_implementors_types_paths,
			implementors_introduced_generics: local_implementors_introduced_generics,
		} = helpers::resolve_implementors_for_struct(
			attrs.0.iter().filter_map(|macro_attr| match macro_attr {
				MacroAttr::LocalImplementor(path) => Some(path),
				_ => None,
			}),
			&struct_,
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

		let mut implementors_introduced_generics = crate_implementors_introduced_generics;
		implementors_introduced_generics.extend(local_implementors_introduced_generics);

		let mut new_struct_fields = Punctuated::new();

		if !already_expanded {
			crate_implementors_idents.iter().zip(crate_implementors_types_paths).for_each(
				|(name, implementor)| {
					new_struct_fields.push(parse_quote!(#struct_vis #name: #implementor))
				},
			);

			local_implementors_idents.iter().zip(local_implementors_types_paths).for_each(
				|(name, implementor)| {
					new_struct_fields.push(parse_quote!(#struct_vis #name: #implementor))
				},
			);
		}

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
			implementors_introduced_generics,
			new_struct_fields,
		})
	}
}
