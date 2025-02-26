// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::parse::{MacroImplParsed, MacroParsed};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
	parse_quote, punctuated::Punctuated, token::Brace, Field, Fields, FieldsNamed, GenericParam,
	Ident, Index, Lifetime, LifetimeParam, Token, Type,
};

pub(crate) fn expand_finder(parsed: MacroParsed) -> TokenStream {
	let MacroParsed {
		crate_implementors_idents,
		local_implementors_idents,
		mut struct_,
		already_expanded,
		impl_from,
		one,
		implementors_count,
		crate_implementors_indexes,
		local_implementors_indexes,
		generics_idents,
		where_clause,
		new_struct_fields,
	} = parsed;

	let struct_vis = &struct_.vis;
	let struct_name = &struct_.ident;

	let finder_wrapper_name =
		Ident::new(&(struct_.ident.to_string() + "FinderWrapper"), Span::call_site());

	let finder_lifetime: Lifetime = parse_quote! {'finder};

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

	let mut impl_from_block = quote! {};

	if impl_from {
		let fields_names: Vec<&Ident> = struct_fields
			.iter()
			.map(|field| field.ident.as_ref().expect("Named field has ident; qed;"))
			.collect();

		let fields_types: Punctuated<&Type, Token![,]> =
			struct_fields.iter().map(|field| &field.ty).collect();
		let tuple_indexes: Vec<Index> =
			struct_fields.iter().enumerate().map(|(index, _)| Index::from(index)).collect();

		impl_from_block = quote! {
			impl<#generics_idents> From<(#fields_types)> for #struct_name<#generics_idents> #where_clause{
				fn from(tuple: (#fields_types)) -> Self{
					Self{ #(#fields_names: tuple.#tuple_indexes),* }
				}
			}
		};

		struct_.attrs.push(parse_quote!(#[rust_writer_procedural::already_impl_from]));
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
					self.0.found[#local_implementors_indexes] = self.0.finder.#local_implementors_idents.clone().find(file);
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

pub(crate) fn expand_impl_finder(
	visit_lifetime: LifetimeParam,
	parsed: MacroImplParsed,
) -> TokenStream {
	let MacroImplParsed { struct_, generics_idents, where_clause } = parsed;

	let struct_name = &struct_.ident;

	let visit_lifetime = GenericParam::Lifetime(visit_lifetime);

	let visit_lifetime_gen = if generics_idents.iter().any(|generic| generic == &visit_lifetime) {
		quote! {}
	} else {
		quote! {#visit_lifetime,}
	};

	let impl_finder = quote! {
		impl<#visit_lifetime_gen #generics_idents>
		#struct_name<#generics_idents>
		#where_clause
		{
			fn find(&mut self, file: &#visit_lifetime syn::File) -> bool{
				self.visit_file(file);
				self.found.iter().all(|&x| x)
			}
		}
	};

	quote! {
		#impl_finder
	}
}
