// SPDX-License-Identifier: GPL-3.0

use crate::{
	helpers,
	parse::{MacroFinderMutatorParsed, MacroLocalParsed},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, Ident, Index,
	Lifetime, Token, Type,
};

pub(crate) fn expand_mutator(parsed: MacroFinderMutatorParsed) -> TokenStream {
	let MacroFinderMutatorParsed {
		crate_implementors_idents,
		local_implementors_idents,
		mut struct_,
		already_expanded,
		impl_from,
		one,
		implementors_count,
		crate_implementors_indexes,
		local_implementors_indexes,
		implementors_introduced_generics,
		new_struct_fields,
	} = parsed;

	let mutator_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "MutatorWrapper"), Span::call_site());

	let mutator_lifetime: Lifetime = parse_quote! {'mutator};

	helpers::add_new_implementors_generics(&mut struct_, implementors_introduced_generics);

	let (generics_declarations, generics_idents, where_clause) =
		rustilities::parsing::extract_generics(&struct_.generics);

	let where_clause = where_clause.unwrap_or(parse_quote! {where});

	let mut impl_from_block = quote! {};

	if !already_expanded {
		struct_.attrs.push(parse_quote!(#[rust_writer::ast::already_expanded]));
		struct_.attrs.push(parse_quote!(#[derive(Debug, Clone)]));
		// Expand struct fields and return a reference to the inner fields
		let struct_fields: &Punctuated<Field, Token![,]> = match struct_.fields {
			Fields::Unit => {
				struct_.fields = Fields::Named(FieldsNamed {
					brace_token: Brace::default(),
					named: new_struct_fields.clone(),
				});
				&new_struct_fields
			},
			Fields::Named(ref mut fields) => {
				fields.named.extend(new_struct_fields);
				&fields.named
			},
			_ => unreachable!("Parser doesn't allow Unnamed fields; qed;"),
		};

		if impl_from {
			let fields_names: Vec<&Ident> = struct_fields
				.iter()
				.map(|field| field.ident.as_ref().expect("Named field has ident; qed;"))
				.collect();

			let fields_types: Punctuated<&Type, Token![,]> =
				struct_fields.iter().map(|field| &field.ty).collect();
			let tuple_indexes: Vec<Index> =
				struct_fields.iter().enumerate().map(|(index, _)| Index::from(index)).collect();

			let struct_name = struct_.ident.clone();

			impl_from_block = quote! {
				impl<#generics_declarations> From<(#fields_types)> for #struct_name<#generics_idents> #where_clause{
					fn from(tuple: (#fields_types)) -> Self{
						Self{ #(#fields_names: tuple.#tuple_indexes),* }
					}
				}
			};
			helpers::remove_impl_from_attr(&mut struct_);
		}
	}

	let struct_vis = &struct_.vis;
	let struct_name = &struct_.ident;

	let mutator_wrapper = quote! {
		#[derive(Debug, Clone)]
		#struct_vis struct #mutator_wrapper_name<#mutator_lifetime, #generics_declarations>(
			#struct_vis rust_writer::ast::mutator::Mutator<
				#mutator_lifetime,
				#struct_name<#generics_idents>,
				#implementors_count
			>
		) #where_clause;

		impl<#mutator_lifetime, #generics_declarations> From<
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
		impl<#mutator_lifetime, #generics_declarations>
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

	let impl_mutate = quote! {
		impl<#mutator_lifetime, #generics_declarations>
		#mutator_wrapper_name<#mutator_lifetime, #generics_idents>
		#where_clause
		{
			fn mutate(
				&mut self,
				file: &mut syn::File,
				indexes: Option<&[u32]>,
			) -> Result<(), rust_writer::Error> {
				#(
					match indexes {
						Some(indexes) if !indexes.contains(&#crate_implementors_indexes) => (),
						_ => {
							let mut mutator = rust_writer::ast::mutator::Mutator::default()
								.to_mutate(&self.0.mutator.#crate_implementors_idents);
							mutator.visit_file_mut(file);
							self.0.mutated[#crate_implementors_indexes] =
								mutator.mutated.iter().all(|&x| x);
						}
					}
				)*

				#(
					match indexes {
						Some(indexes) if !indexes.contains(&#local_implementors_indexes) => (),
						_ => {
							let mut mutator = self.0.mutator.clone();
							mutator.#local_implementors_idents.visit_file_mut(file);
							self.0.mutated[#local_implementors_indexes] =
								mutator.#local_implementors_idents.mutated.iter().all(|&x| x);
						}
					}
				)*

				if self
					.0
					.mutated
					.iter()
					.enumerate()
					.filter(|(index, _)| match indexes {
						Some(indexes) if !indexes.contains(&(*index as u32)) => false,
						_ => true,
					})
					.all(|(_, &x)| x)
				{
					Ok(())
				} else {
					Err(rust_writer::Error::Descriptive(format!(
						"Cannot mutate using Mutator: {:?}",
						self.0.mutator
					)))
				}
			}
		}
	};

	quote! {
		#struct_
		#impl_from_block
		#mutator_wrapper
		#impl_to_mutate
		#impl_mutate
	}
}

pub(crate) fn expand_local_mutator(parsed: MacroLocalParsed) -> TokenStream {
	let MacroLocalParsed { struct_, generics_idents, where_clause, generics_declarations } = parsed;

	let struct_name = &struct_.ident;

	let impl_mutate = quote! {
		impl<#generics_declarations>
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

			fn mutator_reset(&mut self) {
				self.mutated = self.mutated
					.iter()
					.map(|_| false)
					.collect::<Vec<bool>>()
					.try_into()
					.expect("The Vec has the correct length by construction; qed;");
			}
		}
	};

	quote! {
		#impl_mutate
	}
}
