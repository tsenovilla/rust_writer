// SPDX-License-Identifier: GPL-3.0

use crate::{
	helpers::{self, ResolvedImplementors},
	mutator::parse::{ImplMutatorDef, MutatorDef},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, Ident, Index,
	Lifetime, Token, Type,
};

pub(crate) fn expand_mutator(def: MutatorDef) -> TokenStream {
	let MutatorDef {
		crate_implementors,
		local_implementors,
		mut struct_,
		already_expanded,
		impl_from,
	} = def;

	let ResolvedImplementors {
		new_implementors_idents: new_crate_implementors_idents,
		implementors_idents: crate_implementors_idents,
		new_implementors: new_crate_implementors,
	} = helpers::resolve_implementors_for_struct(crate_implementors.iter(), &mut struct_);

	let ResolvedImplementors {
		new_implementors_idents: new_local_implementors_idents,
		implementors_idents: local_implementors_idents,
		new_implementors: new_local_implementors,
	} = helpers::resolve_implementors_for_struct(local_implementors.iter(), &mut struct_);

	let struct_vis = &struct_.vis;
	let struct_name = &struct_.ident;

	let mutator_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "MutatorWrapper"), Span::call_site());

	let mutator_lifetime: Lifetime = parse_quote! {'mutator};
	let one = Index::from(1);

	let implementors_count = Index::from(crate_implementors.len() + local_implementors.len());

	let crate_implementors_indexes: Vec<Index> =
		(0..crate_implementors.len()).map(Index::from).collect();

	let local_implementors_indexes: Vec<Index> = (crate_implementors.len()..
		crate_implementors.len() + local_implementors.len())
		.map(Index::from)
		.collect();

	let (generics_idents, where_clause) = rustilities::parsing::extract_generics(&struct_.generics);

	let where_clause = where_clause.unwrap_or(parse_quote! {where});

	let mut new_struct_fields: Vec<Field> = new_crate_implementors_idents
		.iter()
		.zip(new_crate_implementors)
		.map(|(name, implementor)| parse_quote!(#struct_vis #name: #implementor))
		.collect();

	new_local_implementors_idents.iter().zip(new_local_implementors).for_each(
		|(name, implementor)| new_struct_fields.push(parse_quote!(#struct_vis #name: #implementor)),
	);

	match struct_.fields {
		Fields::Unit => {
			let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
			fields.extend(new_struct_fields);
			struct_.fields =
				Fields::Named(FieldsNamed { brace_token: Brace::default(), named: fields });
		},
		Fields::Named(ref mut fields) => fields.named.extend(new_struct_fields),
		_ => unreachable!("Parser doesn't allow Unnamed fields; qed;"),
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
			};

			struct_.attrs.push(parse_quote!(#[rust_writer_procedural::already_impl_from]));
		}
	}

	let mutator_wrapper = quote! {
		#[derive(Debug, Clone)]
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
		for rust_writer::ast::mutator::Mutator<'_, rust_writer::ast::mutator::EmptyMutator, #one>
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
						.to_mutate(&self.0.mutator.#crate_implementors_idents);
					mutator.visit_file_mut(file);
					self.0.mutated[#crate_implementors_indexes] = mutator.mutated.iter().all(|&x| x);
				)*

				#(
					self.0.mutator.#local_implementors_idents.clone().visit_file_mut(file);
					self.0.mutated[#local_implementors_indexes] = self.0.mutator
						.#local_implementors_idents.mutated.iter().all(|&x| x);
				)*
			}
		}
	};

	let impl_mutate = quote! {
		impl<#mutator_lifetime, #generics_idents>
		#mutator_wrapper_name<#mutator_lifetime, #generics_idents>
		#where_clause
		{
			fn mutate(&mut self, file: &mut syn::File) -> Result<(), rust_writer::Error>{
				self.visit_file_mut(file);

				if self.0.mutated.iter().all(|&x| x){
					Ok(())
				} else {
					Err(rust_writer::Error::Descriptive(format!("Cannot mutate using Mutator: {:?}", self.0.mutator)))
				}

			}
		}
	};

	if !already_expanded {
		struct_.attrs.push(parse_quote!(#[rust_writer_procedural::already_expanded]));
		struct_.attrs.push(parse_quote!(#[derive(Debug, Clone)]));
	}

	quote! {
		#[rust_writer_procedural::already_expanded]
		#struct_
		#impl_from_block
		#mutator_wrapper
		#impl_to_mutate
		#impl_visit_mut
		#impl_mutate
	}
}

pub(crate) fn expand_impl_mutator(def: ImplMutatorDef) -> TokenStream {
	let ImplMutatorDef { struct_ } = def;

	let struct_name = &struct_.ident;

	let (generics_idents, where_clause) = rustilities::parsing::extract_generics(&struct_.generics);

	let where_clause = where_clause.unwrap_or(parse_quote! {where});

	let impl_mutate = quote! {
		impl<#generics_idents>
		#struct_name<#generics_idents>
		#where_clause
		{
			fn mutate(&mut self, file: &mut syn::File) -> Result<(), rust_writer::Error>{
				self.visit_file_mut(file);

				if self.mutated.iter().all(|&x| x){
					Ok(())
				} else {
					Err(rust_writer::Error::Descriptive(format!("Cannot mutate using Mutator: {:?}", self)))
				}
			}
		}
	};

	quote! {
		#impl_mutate
	}
}
