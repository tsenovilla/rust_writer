// SPDX-License-Identiier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, Item, ItemMod};

/// This implementor targets any item inside a module.
#[derive(Debug, Clone)]
pub struct ItemToMod<'a> {
	/// The module's name.
	pub mod_name: &'a str,
	/// The target item.
	pub item: Item,
}

impl<'a> From<(&'a str, Item)> for ItemToMod<'a> {
	fn from(tuple: (&'a str, Item)) -> Self {
		Self { mod_name: tuple.0, item: tuple.1 }
	}
}

impl<'a> ToFind<'a, ItemToMod<'a>, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a ItemToMod<'a>) -> Finder<'a, ItemToMod<'a>, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, ItemToMod<'a>, 1> {
	fn visit_item_mod(&mut self, item_mod: &'a ItemMod) {
		match item_mod.content {
			Some((_, ref items))
				if item_mod.ident == self.finder.mod_name && items.contains(&self.finder.item) =>
				self.found[0] = true,
			_ => (),
		}
	}
}

impl<'a> ToMutate<'a, ItemToMod<'a>, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a ItemToMod<'a>) -> Mutator<'a, ItemToMod<'a>, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl<'a> VisitMut for Mutator<'a, ItemToMod<'a>, 1> {
	fn visit_item_mod_mut(&mut self, item_mod: &mut ItemMod) {
		match item_mod.content {
			Some((_, ref mut items)) if item_mod.ident == self.mutator.mod_name => {
				self.mutated[0] = true;
				items.push(self.mutator.item.clone());
			},
			_ => (),
		}
	}
}
