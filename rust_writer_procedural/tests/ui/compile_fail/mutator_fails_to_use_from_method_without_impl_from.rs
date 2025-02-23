// SPDX-License-Identifier: GPL-3.0

use rust_writer_procedural::mutator;
use rust_writer::{ast::{mutator::ToMutate, implementors::{ItemToTrait, ItemToImpl}}};
use syn::{parse_quote, TraitItem, ImplItem};

#[mutator(ItemToTrait<'a>,ItemToImpl<'a>)]
struct SomeStruct; 

fn main(){
		let item_to_trait: ItemToTrait =
			("MyTrait", TraitItem::Type(parse_quote! {type Type1: From<String>;})).into();

		let item_to_impl: ItemToImpl = (
			"SomeTrait",
			"SomeImplementor",
			ImplItem::Fn(parse_quote! {
			fn some_func(&self) -> bool{
						true
					 }
			  }),
		)
		.into();

    let _some_struct: SomeStruct = (item_to_trait, item_to_impl).into();

}
