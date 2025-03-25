// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use syn::{Attribute, ImplItem, Item, TraitItem};

pub(crate) trait AttrsMut {
	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>>;
}

pub(crate) fn item_without_docs<T: AttrsMut + Clone>(item: &T) -> T {
	let mut output = item.clone();
	if let Some(attrs) = output.attrs_mut() {
		attrs.retain(|attr| !attr.path().is_ident("doc"));
	}
	output
}

impl AttrsMut for Item {
	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			Item::Const(item) => Some(&mut item.attrs),
			Item::Enum(item) => Some(&mut item.attrs),
			Item::ExternCrate(item) => Some(&mut item.attrs),
			Item::Fn(item) => Some(&mut item.attrs),
			Item::ForeignMod(item) => Some(&mut item.attrs),
			Item::Impl(item) => Some(&mut item.attrs),
			Item::Macro(item) => Some(&mut item.attrs),
			Item::Mod(item) => Some(&mut item.attrs),
			Item::Static(item) => Some(&mut item.attrs),
			Item::Struct(item) => Some(&mut item.attrs),
			Item::Trait(item) => Some(&mut item.attrs),
			Item::TraitAlias(item) => Some(&mut item.attrs),
			Item::Type(item) => Some(&mut item.attrs),
			Item::Union(item) => Some(&mut item.attrs),
			Item::Use(item) => Some(&mut item.attrs),
			_ => None,
		}
	}
}

impl AttrsMut for ImplItem {
	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			ImplItem::Const(item) => Some(&mut item.attrs),
			ImplItem::Fn(item) => Some(&mut item.attrs),
			ImplItem::Type(item) => Some(&mut item.attrs),
			ImplItem::Macro(item) => Some(&mut item.attrs),
			_ => None,
		}
	}
}

impl AttrsMut for TraitItem {
	fn attrs_mut(&mut self) -> Option<&mut Vec<Attribute>> {
		match self {
			TraitItem::Const(item) => Some(&mut item.attrs),
			TraitItem::Fn(item) => Some(&mut item.attrs),
			TraitItem::Type(item) => Some(&mut item.attrs),
			TraitItem::Macro(item) => Some(&mut item.attrs),
			_ => None,
		}
	}
}
