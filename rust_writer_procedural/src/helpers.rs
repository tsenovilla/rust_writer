// SPDX-License-Identifier: GPL-3.0

use proc_macro2::Span;
use syn::{parse_quote, Fields, GenericParam, Ident, ItemStruct, Path, PathArguments};

pub(crate) fn remove_impl_from_attr(struct_: &mut ItemStruct) {
	struct_.attrs = struct_
		.attrs
		.clone()
		.into_iter()
		.filter(|attr| !attr.path().is_ident("impl_from"))
		.collect()
}

pub(crate) struct ResolvedImplementors {
	pub(crate) new_implementors_idents: Vec<Ident>,
	pub(crate) new_implementors: Vec<Path>,
	pub(crate) implementors_idents: Vec<Ident>,
}

pub(crate) fn resolve_implementors_for_struct<'a, T>(
	iter: T,
	struct_: &mut ItemStruct,
) -> ResolvedImplementors
where
	T: Iterator<Item = &'a Path>,
{
	let mut new_implementors_idents = Vec::new();
	let mut new_implementors = Vec::new();
	let mut implementors_idents = Vec::new();

	for implementor in iter {
		let last_implementor_segment = implementor
			.segments
			.last()
			.expect("At this point, implementors are valid paths; qed;");

		let ident = Ident::new(
			&last_implementor_segment.ident.to_string().to_lowercase(),
			Span::call_site(),
		);

		match struct_.fields {
			Fields::Named(ref fields)
				if fields.named.iter().any(|field| {
					field.ident.as_ref().expect("Named fields have idents; qed;") == &ident
				}) =>
			{
				implementors_idents.push(ident);
				continue
			},
			_ => {
				new_implementors_idents.push(ident.clone());
				new_implementors.push(implementor.clone());
				implementors_idents.push(ident);
			},
		}

		if let PathArguments::AngleBracketed(ref generics) = last_implementor_segment.arguments {
			generics.args.iter().for_each(|argument| {
				let generic_param: GenericParam = parse_quote!(#argument);
				if !struct_.generics.params.iter().any(|generic| generic == &generic_param) {
					match generic_param {
						GenericParam::Lifetime(_) => {
							struct_.generics.params.insert(0, generic_param);
						},
						_ => {
							struct_.generics.params.push(generic_param);
						},
					}
				}
			})
		}
	}

	ResolvedImplementors { new_implementors_idents, new_implementors, implementors_idents }
}
