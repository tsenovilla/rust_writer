// SPDX-License-Identifier: GPL-3.0

use crate::{
	finder::parse::{FinderDef, ImplFinderDef},
	helpers::{self, ResolvedImplementors},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, Ident, Index,
	Lifetime, Token, Type,
};

pub(crate) fn expand_finder(def: FinderDef) -> TokenStream {
	let FinderDef {
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

	let finder_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "FinderWrapper"), Span::call_site());

	let finder_lifetime: Lifetime = parse_quote! {'finder};
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

	let finder_wrapper = quote! {
		#[derive(Debug, Clone)]
		#struct_vis struct #finder_wrapper_name<#finder_lifetime, #generics_idents>(
			#struct_vis rust_writer::ast::finder::Finder<
				#finder_lifetime,
				#struct_name<#generics_idents>,
				#implementors_count
			>
		) #where_clause;

		impl<#finder_lifetime, #generics_idents> From<
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
		impl<#finder_lifetime, #generics_idents>
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

	let impl_visit = quote! {
		impl<#finder_lifetime, #generics_idents>
		syn::visit::Visit<#finder_lifetime>
		for #finder_wrapper_name<#finder_lifetime, #generics_idents>
		#where_clause
		{
			fn visit_file(&mut self, file: &#finder_lifetime syn::File){
				#(
					let mut finder = rust_writer::ast::finder::Finder::default()
						.to_find(&self.0.finder.#crate_implementors_idents);
					self.0.found[#crate_implementors_indexes] = finder.find(file);
				)*

				#(
					self.0.finder[#local_implementors_indexes] = self.0.#local_implementors_idents.find(file);
				)*
			}
		}
	};

	let impl_find = quote! {
		impl<#finder_lifetime, #generics_idents>
		#finder_wrapper_name<#finder_lifetime, #generics_idents>
		#where_clause
		{
			fn find(&mut self, file: &#finder_lifetime syn::File) -> bool{
				self.visit_file(file);
				self.0.found.iter().all(|&x| x)
			}
		}
	};

	if !already_expanded {
		struct_.attrs.push(parse_quote!(#[rust_writer_procedural::already_expanded]));
		struct_.attrs.push(parse_quote!(#[derive(Debug, Clone)]));
	}

	quote! {
		#struct_
		#impl_from_block
		#finder_wrapper
		#impl_to_find
		#impl_visit
		#impl_find
	}
}

pub(crate) fn expand_impl_finder(def: ImplFinderDef) -> TokenStream {
	let ImplFinderDef { struct_ } = def;

	let struct_name = &struct_.ident;

	let (generics_idents, where_clause) = rustilities::parsing::extract_generics(&struct_.generics);

	let where_clause = where_clause.unwrap_or(parse_quote! {where});

	let impl_finder = quote! {
		impl<#generics_idents>
		#struct_name<#generics_idents>
		#where_clause
		{
			fn find(&self, file: &syn::File) -> bool{
				self.visit_file(file);
				self.found.iter().all(|&x| x)
			}
		}
	};

	quote! {
		#impl_finder
	}
}
