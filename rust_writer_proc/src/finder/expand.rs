// SPDX-License-Identifier: GPL-3.0

use crate::{
	helpers,
	parse::{MacroFinderMutatorParsed, MacroLocalParsed},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, GenericParam,
	Ident, Index, Lifetime, LifetimeParam, Token, Type,
};

pub(crate) fn expand_finder(parsed: MacroFinderMutatorParsed) -> TokenStream {
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

	let finder_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "FinderWrapper"), Span::call_site());

	let finder_lifetime: Lifetime = parse_quote! {'finder};

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

	let finder_wrapper = quote! {
		#[derive(Debug, Clone)]
		#struct_vis struct #finder_wrapper_name<#finder_lifetime, #generics_declarations>(
			#struct_vis rust_writer::ast::finder::Finder<
				#finder_lifetime,
				#struct_name<#generics_idents>,
				#implementors_count
			>
		) #where_clause;

		impl<#finder_lifetime, #generics_declarations> From<
			rust_writer::ast::finder::Finder<#finder_lifetime, #struct_name<#generics_idents>, #implementors_count>
		> for #finder_wrapper_name<#finder_lifetime, #generics_idents> #where_clause{
			#struct_vis fn from(input: rust_writer::ast::finder::Finder<
				#finder_lifetime,
				#struct_name<#generics_idents>,
				#implementors_count
			>) -> Self {
				Self(input)
			}
		}
	};

	let impl_to_find = quote! {
		impl<#finder_lifetime, #generics_declarations>
		rust_writer::ast::finder::ToFind<#finder_lifetime, #struct_name<#generics_idents>, #implementors_count>
		for rust_writer::ast::finder::Finder<'_, rust_writer::ast::finder::EmptyFinder, #one>
		#where_clause
		{
			fn to_find(self, finder: &#finder_lifetime #struct_name<#generics_idents>)
			->
			rust_writer::ast::finder::Finder<#finder_lifetime, #struct_name<#generics_idents>,#implementors_count> {
				rust_writer::ast::finder::Finder {
					found: [false; #implementors_count],
					finder
				}
			}
		}
	};

	let impl_wrapper = quote! {
		impl<#finder_lifetime, #generics_declarations>
		#finder_wrapper_name<#finder_lifetime, #generics_idents>
		#where_clause
		{
			fn find(&mut self, file: &#finder_lifetime syn::File, indexes: Option<&[u32]>) -> bool {
				#(
					match indexes {
						Some(indexes) if !indexes.contains(&#crate_implementors_indexes) => (),
						_ => {
							let mut finder = rust_writer::ast::finder::Finder::default()
								.to_find(&self.0.finder.#crate_implementors_idents);
							self.0.found[#crate_implementors_indexes] = finder.find(file);
						}
					}
				)*

				#(
					match indexes {
						Some(indexes) if !indexes.contains(&#local_implementors_indexes) => (),
						_ => {
							self.0.found[#local_implementors_indexes] =
								self.0.finder.#local_implementors_idents.clone().find(file);
						}
					}
				)*

				self.0.found
					.iter()
					.enumerate()
					.filter(|(index, _)| match indexes {
						Some(indexes) if !indexes.contains(&(*index as u32)) => false,
						_ => true,
					})
					.all(|(_, &x)| x)
			}

			fn get_missing_indexes(&self) -> Option<Vec<u32>> {
				let missing_indexes: Vec<u32> = self.0.found
					.iter()
					.enumerate()
					.filter_map(|(index, found)| {
						if !found {
							Some(index as u32)
						} else {
							None
						}
					})
					.collect();

				if missing_indexes.len() > 0 {
					Some(missing_indexes)
				} else {
					None
				}
			}
		}
	};

	quote! {
		#struct_
		#impl_from_block
		#finder_wrapper
		#impl_to_find
		#impl_wrapper
	}
}

pub(crate) fn expand_local_finder(
	visit_lifetime: LifetimeParam,
	parsed: MacroLocalParsed,
) -> TokenStream {
	let MacroLocalParsed { struct_, generics_declarations, generics_idents, where_clause } = parsed;

	let struct_name = &struct_.ident;

	let visit_lifetime_ident = &visit_lifetime.lifetime;
	let visit_lifetime_ident: GenericParam = parse_quote!(#visit_lifetime_ident);
	let visit_lifetime = GenericParam::Lifetime(visit_lifetime);

	let visit_lifetime_declaration =
		if generics_declarations.iter().any(|generic| generic == &visit_lifetime) {
			quote! {}
		} else {
			quote! {#visit_lifetime,}
		};

	let local_finder = quote! {
		impl<#visit_lifetime_declaration #generics_declarations>
		#struct_name<#generics_idents>
		#where_clause
		{
			fn find(&mut self, file: &#visit_lifetime_ident syn::File) -> bool {
				self.visit_file(file);
				self.found.iter().all(|&x| x)
			}

			fn finder_reset(&mut self) {
				self.found = self.found
					.iter()
					.map(|_| false)
					.collect::<Vec<bool>>()
					.try_into()
					.expect("The Vec has the correct length by construction; qed;");
			}
		}
	};

	quote! {
		#local_finder
	}
}
