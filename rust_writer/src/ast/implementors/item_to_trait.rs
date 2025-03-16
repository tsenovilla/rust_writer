// SPDX-License-Identifier: GPL-3.0

#[cfg(test)]
mod tests;

use crate::ast::{
	finder::{EmptyFinder, Finder, ToFind},
	mutator::{EmptyMutator, Mutator, ToMutate},
};
use syn::{visit::Visit, visit_mut::VisitMut, ItemTrait, TraitItem};

#[derive(Debug, Clone)]
pub struct ItemToTrait<'a> {
	pub trait_name: &'a str,
	pub item_trait: TraitItem,
}

impl<'a> From<(&'a str, TraitItem)> for ItemToTrait<'a> {
	fn from(tuple: (&'a str, TraitItem)) -> Self {
		Self { trait_name: tuple.0, item_trait: tuple.1 }
	}
}

impl<'a> ToFind<'a, ItemToTrait<'a>, 1> for Finder<'a, EmptyFinder, 1> {
	fn to_find(self, finder: &'a ItemToTrait<'a>) -> Finder<'a, ItemToTrait<'a>, 1> {
		Finder { found: self.found, finder }
	}
}

impl<'a> Visit<'a> for Finder<'a, ItemToTrait<'a>, 1> {
	fn visit_item_trait(&mut self, item_trait: &'a ItemTrait) {
		if item_trait.ident == self.finder.trait_name &&
			item_trait.items.contains(&self.finder.item_trait)
		{
			self.found[0] = true;
		}
	}
}

impl<'a> ToMutate<'a, ItemToTrait<'a>, 1> for Mutator<'_, EmptyMutator, 1> {
	fn to_mutate(self, mutator: &'a ItemToTrait<'a>) -> Mutator<'a, ItemToTrait<'a>, 1> {
		Mutator { mutated: self.mutated, mutator }
	}
}

impl<'a> VisitMut for Mutator<'a, ItemToTrait<'a>, 1> {
	fn visit_item_trait_mut(&mut self, item_trait: &mut ItemTrait) {
		if item_trait.ident == self.mutator.trait_name {
			self.mutated[0] = true;
			item_trait.items.push(self.mutator.item_trait.clone());
		}
	}
}
