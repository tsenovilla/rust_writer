// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::Error;
use std::{cmp::PartialEq, fmt::Debug};
use syn::{visit_mut::VisitMut, File, Item, ItemTrait, TraitItem};

pub trait ToModify<T: Debug> {
	fn to_modify(self, modifiers: T) -> Expander<T>;
}

#[derive(PartialEq, Debug)]
pub struct NotModifiers;

pub struct Expander<T: Debug> {
	modified: bool,
	modifiers: T,
}

impl Default for Expander<NotModifiers> {
	fn default() -> Self {
		Self { modified: false, modifiers: NotModifiers }
	}
}

impl<T: PartialEq<NotModifiers> + Debug> Expander<T>
where
	Expander<T>: VisitMut,
{
	pub fn expand(mut self, ast: &mut File) -> Result<(), Error> {
		if self.modifiers == NotModifiers {
			return Err(Error::Descriptive("Trying to expand without defined modifiers".to_owned()));
		}

		self.visit_file_mut(ast);

		if self.modified {
			Ok(())
		} else {
			Err(Error::Descriptive(format!("Cannot expand {:?}", self.modifiers)))
		}
	}
}

#[derive(Debug)]
pub struct TypeToTrait {
	trait_name: String,
	type_: TraitItem,
}

impl From<(String, TraitItem)> for TypeToTrait {
	fn from(tuple: (String, TraitItem)) -> Self {
		Self { trait_name: tuple.0, type_: tuple.1 }
	}
}

impl PartialEq<NotModifiers> for TypeToTrait {
	fn eq(&self, _other: &NotModifiers) -> bool {
		false
	}
}

impl ToModify<TypeToTrait> for Expander<NotModifiers> {
	fn to_modify(self, modifiers: TypeToTrait) -> Expander<TypeToTrait> {
		Expander { modified: self.modified, modifiers }
	}
}

impl VisitMut for Expander<TypeToTrait> {
	fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
		if item_trait.ident == self.modifiers.trait_name {
			self.modified = true;
			item_trait.items.push(self.modifiers.type_.clone());
		}
	}
}

impl VisitMut for Expander<NotModifiers> {}
