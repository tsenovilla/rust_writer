// SPDX-License-Identifier: GPL-3.0

use crate::mutator::parse::MutatorDef;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, GenericParam,
	Ident, Index, Lifetime, PathArguments, Token, Type, WhereClause, WherePredicate,
};

pub(crate) fn expand_mutator(def: MutatorDef) -> TokenStream {
	let MutatorDef { implementors, mut struct_, unit_struct, impl_from } = def;

	let struct_vis = &struct_.vis;
	let struct_name = &struct_.ident;

	let empty_mutator_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "EmptyMutatorWrapper"), Span::call_site());
	let mutator_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "MutatorWrapper"), Span::call_site());

	let mutator_lifetime: Lifetime = parse_quote! {'mutator};
	let one = Index::from(1);

	let mut implementors_generics = Vec::new();
	let mut implementors_lifetimes = Vec::new();

	let implementors_count = Index::from(implementors.len());

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
					match generic_param {
						GenericParam::Lifetime(_) => {
							if !implementors_lifetimes.contains(&generic_param) {
								implementors_lifetimes.push(generic_param);
							}
						},
						_ =>
							if !implementors_generics.contains(&generic_param) {
								implementors_generics.push(generic_param);
							},
					}
				})
			}

			Ident::new(
				&last_implementor_segment.ident.to_string().to_lowercase(),
				Span::call_site(),
			)
		})
		.collect();

	implementors_lifetimes.into_iter().for_each(|lifetime| {
		struct_.generics.params.insert(0, lifetime);
	});

	struct_.generics.params.extend(implementors_generics);

	let mut where_clauses: Punctuated<WherePredicate, Token![,]> = Punctuated::new();
	let generics_idents: Punctuated<GenericParam, Token![,]> = struct_
		.generics
		.params
		.iter()
		.map(|item| {
			if let GenericParam::Type(generic_type) = item {
				let ident = &generic_type.ident;
				let bounds = &generic_type.bounds;
				where_clauses.push(parse_quote! {#ident: #bounds});
				GenericParam::Type(parse_quote! { #ident })
			} else {
				item.clone()
			}
		})
		.collect();

	let mut where_clause: WhereClause =
		struct_.generics.where_clause.clone().unwrap_or(parse_quote! {where});

	where_clause.predicates.extend(where_clauses);

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
	} else if let Fields::Named(ref mut fields) = struct_.fields {
		fields.named.extend(new_struct_fields);
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
				impl<#generics_idents> From<(#fields_types)> for #struct_name<#generics_idents> #where_clause{
					fn from(tuple: (#fields_types)) -> Self{
						Self{ #(#fields_names: tuple.#tuple_indexes),* }
					}
				}
			}
		}
	}

	let mutator_wrappers = quote! {
		#struct_vis struct #empty_mutator_wrapper_name<#mutator_lifetime>(
		  rust_writer::ast::mutator::Mutator<
			  #mutator_lifetime,
			  rust_writer::ast::mutator::EmptyMutator,
			  #one
		  >
	   );

	  impl<#mutator_lifetime> From<
	   rust_writer::ast::mutator::Mutator<#mutator_lifetime,rust_writer::ast::mutator::EmptyMutator, #one>
	  > for #empty_mutator_wrapper_name<#mutator_lifetime>{
		  #struct_vis fn from(input: rust_writer::ast::mutator::Mutator<#mutator_lifetime,rust_writer::ast::mutator::EmptyMutator,#one>)
		  -> Self{
			  Self(input)
		  }
	  }

		#struct_vis struct #mutator_wrapper_name<#mutator_lifetime, #generics_idents>(
		  #struct_vis rust_writer::ast::mutator::Mutator<
			  #mutator_lifetime,
			  #struct_name<#generics_idents>,
			  #implementors_count
			>
	   ) #where_clause;

	  impl<#mutator_lifetime, #generics_idents> From<
	   rust_writer::ast::mutator::Mutator<#mutator_lifetime, #struct_name<#generics_idents>, #implementors_count>
	  > for #mutator_wrapper_name<#mutator_lifetime, #generics_idents> #where_clause{
		  #struct_vis fn from(input: rust_writer::ast::mutator::Mutator<
			  #mutator_lifetime,
			  #struct_name<#generics_idents>,
			  #implementors_count
		  >) -> Self {
		   Self(input)
		  }
	  }

	};

	let impl_to_mutate = quote! {
		impl<#mutator_lifetime, #generics_idents>
		rust_writer::ast::mutator::ToMutate<#mutator_lifetime, #struct_name<#generics_idents>, #implementors_count>
		for #empty_mutator_wrapper_name<'_>
	  #where_clause
	  {
			fn to_mutate(self, mutator: &#mutator_lifetime #struct_name<#generics_idents>)
			->
			rust_writer::ast::mutator::Mutator<#mutator_lifetime, #struct_name<#generics_idents>,#implementors_count> {
				rust_writer::ast::mutator::Mutator {
					mutated: [false; #implementors_count],
				  mutator
				}
			}
		}
	};

	let impl_visit_mut = quote! {
		impl<#mutator_lifetime, #generics_idents>
		syn::visit_mut::VisitMut
		for #mutator_wrapper_name<#mutator_lifetime, #generics_idents>
	  #where_clause
	  {
			fn visit_file_mut(&mut self, file: &mut syn::File){
			#(
				  let mut mutator = rust_writer::ast::mutator::Mutator::default()
					  .to_mutate(&self.0.mutator.#implementors_struct_names);
				  mutator.visit_file_mut(file);
				  self.0.mutated[#implementors_indexes] = mutator.mutated.iter().all(|&x| x);
			)*
		 }
		}
	};

	quote! {
		#[rust_writer_procedural::already_expanded]
		#[derive(Debug, Clone)]
		#struct_
		#impl_from_block
	  #mutator_wrappers
		#impl_to_mutate
	  #impl_visit_mut
	}
}
