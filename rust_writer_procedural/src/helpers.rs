// SPDX-License-Identifier: GPL-3.0

use proc_macro2::Span;
use syn::{
	parse_quote, punctuated::Punctuated, Fields, GenericArgument, GenericParam, Ident, ItemStruct,
	Path, PathArguments, Token,
};

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

const UNREACHABLE_MESSAGE: &str =
	"syn::PathArguments cannot be parsed with `const ident:type` in the arguments; qed;";

pub(crate) fn resolve_implementors_for_struct<'a, T>(
	iter: T,
	struct_: &mut ItemStruct,
) -> ResolvedImplementors
where
	T: Iterator<Item = &'a Path>,
{
	let mut new_implementors_idents = Vec::new();
	let mut new_implementors = Vec::new();
	let mut implementors_idents: Vec<Ident> = Vec::new();

	for implementor in iter {
		let mut implementor = implementor.clone();

		let last_implementor_segment = implementor
			.segments
			.last_mut()
			.expect("At this point, implementors are valid paths; qed;");

		let implementor_ident_value = last_implementor_segment.ident.to_string().to_lowercase();

		let ident_count = implementors_idents
			.iter()
			.filter(|&ident| *ident == implementor_ident_value)
			.count();

		let ident = if ident_count > 0 {
			Ident::new(
				&(implementor_ident_value + "_" + &ident_count.to_string()),
				Span::call_site(),
			)
		} else {
			Ident::new(&implementor_ident_value, Span::call_site())
		};

		if let PathArguments::AngleBracketed(ref mut generics) = last_implementor_segment.arguments
		{
			let mut last_implementor_generics_idents: Punctuated<GenericArgument, Token![,]> =
				Punctuated::new();

			generics.args.iter().for_each(|argument| {
				let generic_param: GenericParam = parse_quote!(#argument);
				match generic_param {
					GenericParam::Lifetime(_) => {
						last_implementor_generics_idents.insert(0, parse_quote!(#generic_param));
						if !struct_.generics.params.iter().any(|generic| generic == &generic_param)
						{
							struct_.generics.params.insert(0, generic_param);
						}
					},
					GenericParam::Type(ref generic) => {
						let generic_ident = &generic.ident;
						last_implementor_generics_idents.push(parse_quote!(#generic_ident));
						if !struct_.generics.params.iter().any(|generic| {
							generic == &generic_param ||
                // Support for generics const combines including their ident as a generic and the
                // actual const declaration inside the struct def.
								matches!(generic, GenericParam::Const(inner) if &inner.ident == generic_ident)
						}) {
							struct_.generics.params.push(generic_param);
						}
					},
					GenericParam::Const(_) => unreachable!("{}", UNREACHABLE_MESSAGE),
				}
			});

			generics.args = last_implementor_generics_idents;
		}

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
				new_implementors.push(implementor);
				implementors_idents.push(ident);
			},
		}
	}

	ResolvedImplementors { new_implementors_idents, new_implementors, implementors_idents }
}
