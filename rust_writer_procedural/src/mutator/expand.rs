// SPDX-License-Identifier: GPL-3.0

use crate::mutator::parse::MutatorDef;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, GenericParam,
	Ident, Index, Lifetime, PathArguments, Token, Type,
};

pub(crate) fn expand_mutator(def: MutatorDef) -> TokenStream {
	let MutatorDef { implementors, mut struct_, unit_struct, impl_from } = def;

	let struct_vis = &struct_.vis;
	let struct_name = &struct_.ident;

	let mutator_lifetime: Lifetime = parse_quote! {'mutator};
	let one = Index::from(1);

	let mut implementor_generics = Vec::new();

	let implementors_count = Index::from(implementors.iter().count());

	let implementors_indexes: Vec<Index> =
		implementors.iter().enumerate().map(|(index, _)| Index::from(index)).collect();

	let implementors_struct_names: Vec<Ident> = implementors
		.iter()
		.map(|implementor| {
			let last_implementor_segment = implementor
				.segments
				.last()
				.expect("At this point, implementors are valid paths; qed;");

			if let PathArguments::AngleBracketed(ref generics) = last_implementor_segment.arguments
			{
				generics.args.iter().for_each(|argument| {
					let generic_param: GenericParam = parse_quote!(#argument);
					if !implementor_generics.contains(&generic_param) {
						implementor_generics.push(generic_param);
					}
				})
			}

			Ident::new(
				&last_implementor_segment.ident.to_string().to_lowercase(),
				Span::call_site(),
			)
		})
		.collect();

	struct_.generics.params.extend(implementor_generics);

	let generics_params = &struct_.generics.params;

	let where_clause = struct_
		.generics
		.where_clause
		.as_ref()
		.map(|clause| clause.into_token_stream())
		.unwrap_or(quote! {where});

	let new_struct_fields: Vec<Field> = implementors_struct_names
		.iter()
		.zip(implementors)
		.map(|(name, implementor)| parse_quote!(#struct_vis #name: #implementor))
		.collect();

	if unit_struct {
		let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
		fields.extend(new_struct_fields);
		struct_.fields =
			Fields::Named(FieldsNamed { brace_token: Brace::default(), named: fields });
	} else {
		if let Fields::Named(ref mut fields) = struct_.fields {
			fields.named.extend(new_struct_fields);
		}
	}

	let mut impl_from_block = quote! {};

	if impl_from {
		if let Fields::Named(FieldsNamed { ref named, .. }) = struct_.fields {
			let fields_names: Vec<&Ident> = named
				.iter()
				.map(|field| field.ident.as_ref().expect("Named field has ident; qed;"))
				.collect();

			let fields_types: Punctuated<&Type, Token![,]> =
				named.iter().map(|field| &field.ty).collect();
			let tuple_indexes: Vec<Index> =
				named.iter().enumerate().map(|(index, _)| Index::from(index)).collect();

			impl_from_block = quote! {
				impl<#generics_params> From<(#fields_types)> for #struct_name<#generics_params> #where_clause{
					fn from(tuple: (#fields_types)) -> Self{
						Self{ #(#fields_names: tuple.#tuple_indexes),* }
					}
				}
			}
		}
	}


	let impl_to_mutate = quote! {
		impl<#mutator_lifetime, #generics_params>
		rust_writer::ast::mutator::ToMutate<#mutator_lifetime, #struct_name<#generics_params>, #implementors_count>
		for rust_writer::ast::mutator::Mutator<'_, rust_writer::ast::mutator::EmptyMutator, #one>
	  #where_clause
	  {
			fn to_mutate(self, mutator: &#mutator_lifetime #struct_name<#generics_params>)
			->
			rust_writer::ast::mutator::Mutator<#mutator_lifetime, #struct_name<#generics_params>,#implementors_count> {
				rust_writer::ast::mutator::Mutator {
					mutated: [false; #implementors_count],
				  mutator
				}
			}
		}
	};

	let impl_visit_mut = quote! {
		impl<#mutator_lifetime, #generics_params>
		syn::visit_mut::VisitMut
		for rust_writer::ast::mutator::Mutator<#mutator_lifetime, #struct_name<#generics_params>, #implementors_count>
	  #where_clause
	  {
			fn visit_file_mut(&mut self, file: &mut syn::File){
			#(
				  let mut mutator = rust_writer::ast::mutator::Mutator::default()
					  .to_mutate(&self.mutator.#implementors_struct_names);
				  mutator.visit_file_mut(file);
				  self.mutated[#implementors_indexes] = mutator.mutated.iter().all(|&x| x);
			)*
		 }
		}
	};

	quote! {
		#[rust_writer_procedural::already_expanded]
		#[derive(Debug, Clone)]
		#struct_
		#impl_from_block
		#impl_to_mutate
	  #impl_visit_mut
	}
	.into()
}
